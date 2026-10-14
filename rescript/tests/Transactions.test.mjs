import {test,expect,afterEach} from 'bun:test';
import * as Tx from '../src/program/Transactions.res.mjs';
import * as V1 from '../src/program/V1Transaction.res.mjs';
import * as Client from '../src/Client.res.mjs';
import * as kit from '@solana/kit';
import {ed25519} from '@noble/curves/ed25519';
const original=globalThis.fetch;afterEach(()=>{globalThis.fetch=original});
const ok=r=>{expect(r.TAG).toBe('Ok');return r._0};
const seed=new Uint8Array(32).fill(1),secret=new Uint8Array([...seed,...ed25519.getPublicKey(seed)]);
const payer=kit.getAddressDecoder().decode(secret.slice(32)),blockhash=kit.getAddressDecoder().decode(new Uint8Array(32).fill(7));
const resources={computeUnitLimit:200000,loadedAccountsDataSizeLimit:1048576,priorityFeeLamports:0n};
const context={blockhash,lastValidBlockHeight:100n,resources};
const tx=()=>ok(V1.compile([{programAddress:'11111111111111111111111111111111',accounts:[{address:payer,role:3}],data:new Uint8Array([1])}],payer,context));
const client=()=>Client.make(undefined,undefined,undefined,'https://rpc.test',undefined,undefined,undefined,resources,undefined,undefined);
const feature=()=>({context:{slot:42},value:{owner:'Feature111111111111111111111111111111111111',executable:false,data:[Buffer.from([1,1,0,0,0,0,0,0,0]).toString('base64'),'base64']}});
function rpc(handler){globalThis.fetch=async(_,init)=>{expect(init.credentials).toBe('omit');const request=JSON.parse(init.body);return new Response(JSON.stringify(await handler(request.method,request.params)))}}
test('fee shortfall fails before a wallet prompt',async()=>{
 const c=client();let prompts=0;
 Client.useExternalSigner(c,payer,async()=>new Uint8Array(),async()=>{prompts++;return new Uint8Array()});
 rpc(async method=>({result:{value:method==='getFeeForMessage'?5000:4999}}));
 const result=await Tx.signPrepared(c,tx());expect(result._0.TAG).toBe('InsufficientSolForTransactionFees');expect(prompts).toBe(0);
});
test('prepared message is estimated, signed, simulated and sent once without rebuilding',async()=>{
 const c=client();await Client.useNativeSigner(c,secret);let sends=0,expected,estimated;
 rpc(async(method,params)=>{
  switch(method){case 'getFeeForMessage':estimated=params[0];return {result:{value:5000}};
   case 'getBalance':return {result:{value:9000}};case 'getAccountInfo':return {result:feature()};
   case 'simulateTransaction':expect(params[1]).toMatchObject({sigVerify:true,replaceRecentBlockhash:false});expected=ok(await V1.signature(ok(V1.fromWireBytes(Buffer.from(params[0],'base64'),context))));expect(Buffer.from(estimated,'base64')).toEqual(Buffer.from(V1.messageBytes(tx())));return {result:{context:{slot:9},value:{err:null,unitsConsumed:100,logs:[]}}};
   case 'sendTransaction':sends++;expect(params[1].maxRetries).toBe(0);return {result:expected};
   case 'getSignatureStatuses':return {result:{value:[{slot:10,err:null,confirmationStatus:'confirmed'}]}};
   default:throw Error(method);
  }
 });
 expect(ok(await Tx.signAndSubmitConfirmed(c,tx()))).toEqual({signature:expected,slot:10n});expect(sends).toBe(1);
});
test('ambiguous sends preserve signature and expiry and never retry',async()=>{
 const c=client();await Client.useNativeSigner(c,secret);let sends=0;
 const signed=ok(await V1.sign(tx(),[await kit.createKeyPairFromBytes(secret)]));
 for(const rejection of [null,{error:{code:-32002,message:'Already processed'}},{result:'wrong'}]){
  rpc(async method=>{if(method==='getAccountInfo')return {result:feature()};if(method==='simulateTransaction')return {result:{context:{slot:9},value:{err:null}}};if(method==='sendTransaction'){sends++;if(rejection===null)throw Error('lost');return rejection}throw Error(method)});
  const result=await Tx.submitSigned(c,signed);expect(result._0.TAG).toBe('SubmissionUnknown');expect(result._0.lastValidBlockHeight).toBe(100n);
 }
 expect(sends).toBe(3);
});
test('definite preflight rejection is typed',async()=>{
 const c=client(),signed=ok(await V1.sign(tx(),[await kit.createKeyPairFromBytes(secret)]));
 rpc(async method=>method==='getAccountInfo'?{result:feature()}:method==='simulateTransaction'?{result:{context:{slot:9},value:{err:null}}}:{error:{code:-32002,message:'simulation failed'}});
 expect((await Tx.submitSigned(c,signed))._0.TAG).toBe('SubmissionRejected');
});
test('confirmation requires consecutive expiry samples and searches history',async()=>{
 const c=client();let status=0,height=0,history=0;
 rpc(async(method,params)=>{if(method==='getBlockHeight'){height++;return {result:101}};status++;if(params[1].searchTransactionHistory){history++;return {result:{value:[{slot:88,err:null,confirmationStatus:'finalized'}]}}}return {result:{value:status===2?[{slot:2,err:null,confirmationStatus:'processed'}]:[null]}}});
 expect(ok(await Tx.confirm(c,'signature',100n,0,10)).slot).toBe(88n);expect(height).toBe(3);expect(history).toBe(1);
});
test('confirmation distinguishes expired, timeout, and landed failure',async()=>{
 const c=client();rpc(async method=>({result:method==='getBlockHeight'?101:{value:[null]}}));
 expect((await Tx.confirm(c,'sig',100n,0,4))._0.TAG).toBe('TransactionExpired');
 expect((await Tx.confirm(c,'sig',undefined,0,2))._0.TAG).toBe('ConfirmationTimeout');
 rpc(async()=>({result:{value:[{slot:1,err:{InstructionError:[0,'Custom']},confirmationStatus:'confirmed'}]}}));
 expect((await Tx.confirm(c,'sig',100n,0,2))._0.TAG).toBe('TransactionFailed');
});
