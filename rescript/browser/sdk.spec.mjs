import {test,expect} from '@playwright/test';
import {build} from 'esbuild';
import {createServer} from 'node:http';
import {WebSocketServer} from 'ws';
let server,wss,url,wsUrl,bundle,messages=[];
test.beforeAll(async()=>{
 const built=await build({stdin:{contents:`import * as SDK from './src/BrowserApi.res.mjs'; import * as Http from './src/Http.res.mjs'; import * as Client from './src/Client.res.mjs'; import * as V1 from './src/program/V1Transaction.res.mjs'; import {ed25519} from '@noble/curves/ed25519'; import * as kit from '@solana/kit'; globalThis.sdkTest={SDK,Http,Client,V1,ed25519,kit};`,resolveDir:process.cwd()},bundle:true,platform:'browser',format:'iife',write:false,metafile:true});
 expect(Object.keys(built.metafile.inputs).some(p=>p.includes('node_modules/ws/')||p.endsWith('ServerApi.res.mjs'))).toBe(false);bundle=built.outputFiles[0].text;
 server=createServer((req,res)=>{if(req.url==='/sdk.js'){res.setHeader('Content-Type','application/javascript');res.end(bundle)}else{res.setHeader('Content-Type','text/html');res.end('<script src="/sdk.js"></script>')}});
 wss=new WebSocketServer({server});wss.on('connection',socket=>socket.on('message',m=>messages.push(JSON.parse(m.toString()))));
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));url=`http://127.0.0.1:${server.address().port}`;wsUrl=url.replace('http','ws');
});
test.afterAll(async()=>{if(!wss)return;for(const c of wss.clients)c.terminate();await new Promise(r=>wss.close(r));await new Promise(r=>server.close(r))});
test('browser cookie ownership, blocked server keys, and exact external signing',async({page})=>{
 await page.goto(url);
 const result=await page.evaluate(async()=>{
  const {SDK,Http,Client,V1,ed25519,kit}=sdkTest;
  const c=SDK.makeForEnv('staging');Http.setAuthToken(c.http,'native-token');const key=Http.setServerApiKey(c.http,'secret');
  const seed=new Uint8Array(32).fill(1),pub=ed25519.getPublicKey(seed),wallet=kit.getAddressDecoder().decode(pub);
  const ctx={blockhash:kit.getAddressDecoder().decode(new Uint8Array(32).fill(7)),lastValidBlockHeight:100n,resources:{computeUnitLimit:200000,loadedAccountsDataSizeLimit:1048576,priorityFeeLamports:0n}};
  const unsigned=V1.compile([{programAddress:'11111111111111111111111111111111',accounts:[{address:wallet,role:3}],data:new Uint8Array([1])}],wallet,ctx)._0;
  SDK.useExternalSigner(c,wallet,async message=>ed25519.sign(message,seed),async wire=>{
    const tx=kit.getTransactionDecoder().decode(wire);return new Uint8Array([...tx.messageBytes,...ed25519.sign(tx.messageBytes,seed)]);
  });
  const messageResult=await Client.signMessageBytes(c,new Uint8Array([1,2]));
  const signed=await V1.acceptSignedBytes(unsigned,await c.signingStrategy._0.signTransaction(V1.toWireBytes(unsigned)));
  return {token:Http.authToken(c.http),key:key.TAG,nativeExport:'useNativeSigner' in SDK,loginExport:'login' in SDK.AuthClient,message:messageResult.TAG,signed:signed.TAG};
 });
 expect(result).toEqual({token:undefined,key:'Error',nativeExport:false,loginExport:false,message:'Ok',signed:'Ok'});
});
test('real browser sockets queue subscriptions and recover after stop/restart',async({page})=>{
 messages=[];await page.goto(url);await page.evaluate(wsUrl=>{const {SDK}=sdkTest;const c=SDK.makeForEnv('staging');c.wsUrl=wsUrl;globalThis.connection=SDK.WsClient.connect(c,()=>{});SDK.WsClient.subscribe(connection,{TAG:'WalletDepositBalances',_0:'wallet'});},wsUrl);
 await expect.poll(()=>messages.length).toBe(1);expect(messages[0].params.type).toBe('wallet_deposit_balances');
 await page.evaluate(()=>{sdkTest.SDK.WsClient.stop(connection);sdkTest.SDK.WsClient.restartConnection(connection)});
 await expect.poll(()=>messages.length).toBe(2);await page.evaluate(()=>sdkTest.SDK.WsClient.stop(connection));
});
test('workers reject server API keys and never retain native tokens',async({page})=>{
 await page.goto(url);
 const result=await page.evaluate(url=>new Promise((resolve,reject)=>{
  const source=`importScripts(${JSON.stringify(url+'/sdk.js')}); const {SDK,Http}=sdkTest; const c=SDK.makeForEnv('staging'); Http.setAuthToken(c.http,'native'); postMessage({key:Http.setServerApiKey(c.http,'secret').TAG,token:Http.authToken(c.http)});`;
  const objectUrl=URL.createObjectURL(new Blob([source],{type:'application/javascript'}));
  const worker=new Worker(objectUrl);worker.onmessage=e=>{resolve(e.data);worker.terminate();URL.revokeObjectURL(objectUrl)};worker.onerror=e=>{reject(e.message);worker.terminate();URL.revokeObjectURL(objectUrl)};
 }),url);
 expect(result).toEqual({key:'Error',token:undefined});
});
