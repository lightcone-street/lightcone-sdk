import {test,expect} from 'bun:test';
import {createHash} from 'node:crypto';
import {ed25519} from '@noble/curves/ed25519';
import * as kit from '@solana/kit';
import * as V1 from '../src/program/V1Transaction.res.mjs';
const ok=r=>{expect(r.TAG,JSON.stringify(r,(_,v)=>typeof v==='bigint'?String(v):v)).toBe('Ok');return r._0};
const key=n=>kit.getAddressDecoder().decode(new Uint8Array(32).fill(n));
const keys=await Promise.all([1,2].map(async n=>{const seed=new Uint8Array(32).fill(n);return kit.createKeyPairFromBytes(new Uint8Array([...seed,...ed25519.getPublicKey(seed)]))}));
const payer=await kit.getAddressFromPublicKey(keys[0].publicKey),second=await kit.getAddressFromPublicKey(keys[1].publicKey);
const context={blockhash:key(7),lastValidBlockHeight:100n,resources:{computeUnitLimit:200000,loadedAccountsDataSizeLimit:1048576,priorityFeeLamports:123n}};
const meta=(address,role)=>({address,role});
const ix={programAddress:key(99),accounts:[meta(second,2),meta(key(10),0),meta(payer,3),meta(second,2),meta(key(8),1)],data:new Uint8Array([0,1,2,127,128,255])};
const hash=b=>createHash('sha256').update(b).digest('hex');
test('v1 compilation and all signatures match independent Rust pinned digests',async()=>{
 const merged={...ix,accounts:[...ix.accounts,meta(key(10),1),meta(second,3)]};
 const transfer={programAddress:key(0),accounts:[meta(payer,3),meta(key(9),1)],data:new Uint8Array([2,0,0,0,42,0,0,0,0,0,0,0])};
 const cases=[
 [{computeUnitLimit:1400000,loadedAccountsDataSizeLimit:67108864,priorityFeeLamports:18446744073709551615n,heapSize:262144},[ix],keys,'c5fdf05cc77574469ea3b7b08798722fda2bd225b1b8e35995d42f31e7011290','40156bfa4370bd072ca9000a3c42d1159be9c6c8b32cf598793a276b16f332c3'],
 [{...context.resources,priorityFeeLamports:1001n,heapSize:32768},[ix,merged,transfer],keys,'fdf9fd85a3963251ca654526c1fb6134f4e76ad361aa2b057bb3dfff379f41c4','368fb841224f6273bfc741ac09d88df014050640e8d3f4b31a38c05b1ef04809'],
 [{...context.resources,priorityFeeLamports:0n},[{programAddress:key(99),accounts:[meta(key(99),1),meta(key(98),1)],data:new Uint8Array([1,2])},{programAddress:key(98),accounts:[meta(key(99),0),meta(payer,3)],data:new Uint8Array([3,4])}],[keys[0]],'68e412b10a0ac0b912f9606cf2b62bc158e74ee25a2f5a7a5c20117584cfd7c8','e7629ba9b05db1fd502a408343395bb001aa0fdb48d659c59c1b06983f924307']
 ];
 for(const [resources,instructions,signers,messageHash,wireHash] of cases){
  const tx=ok(V1.compile(instructions,payer,{...context,resources}));
  expect(hash(V1.messageBytes(tx))).toBe(messageHash);
  const signed=ok(await V1.sign(tx,signers));
  expect(hash(V1.toWireBytes(signed))).toBe(wireHash);
  ok(await V1.verifySignatures(signed));
  const copy=V1.toWireBytes(signed);copy.fill(0);ok(await V1.verifySignatures(signed));
 }
});
test('wallets cannot change messages, omit signatures, or import trailing bytes',async()=>{
 const tx=ok(V1.compile([ix],payer,context));
 expect((await V1.acceptSignedBytes(tx,V1.toWireBytes(tx))).TAG).toBe('Error');
 const signed=ok(await V1.sign(tx,keys));
 const bytes=V1.toWireBytes(signed);bytes[bytes.length-1]^=1;
 expect((await V1.acceptSignedBytes(tx,bytes)).TAG).toBe('Error');
 expect(V1.fromWireBytes(new Uint8Array([...V1.toWireBytes(signed),0]),context).TAG).toBe('Error');
 const other=ok(await V1.sign(ok(V1.compile([{...ix,data:new Uint8Array([3])}],payer,context)),keys));
 expect((await V1.acceptSignedBytes(tx,V1.toWireBytes(other))).TAG).toBe('Error');
});
export {ok,key,keys,payer,context,ix};
test('explicit zero priority fee imports without allowing a wallet to rewrite omitted encoding',async()=>{
 const ctx={...context,resources:{...context.resources,priorityFeeLamports:0n}};
 const tx=ok(V1.compile([ix],payer,ctx));
 const message=kit.getCompiledTransactionMessageDecoder().decode(V1.messageBytes(tx));
 message.configMask+=3;message.configValues.unshift({kind:'u64',value:0n});
 const bytes=kit.getCompiledTransactionMessageEncoder().encode(message);
 const signatures=await Promise.all(keys.map(pair=>kit.signBytes(pair.privateKey,bytes)));
 const wire=new Uint8Array([...bytes,...signatures[0],...signatures[1]]);
 ok(await V1.verifySignatures(ok(V1.fromWireBytes(wire,ctx))));
 expect((await V1.acceptSignedBytes(tx,wire)).TAG).toBe('Error');
});
test('resource, signer and legacy boundaries fail before encoding',()=>{
 for(const resources of [{...context.resources,computeUnitLimit:0},{...context.resources,computeUnitLimit:1.5},{...context.resources,heapSize:33000},{...context.resources,priorityFeeLamports:-1n},{...context.resources,loadedAccountsDataSizeLimit:67108865}])expect(V1.compile([ix],payer,{...context,resources}).TAG).toBe('Error');
 expect(V1.compile([],payer,context).TAG).toBe('Error');
 expect(V1.compile([{...ix,programAddress:payer}],payer,context).TAG).toBe('Error');
 expect(V1.compile([{...ix,programAddress:'ComputeBudget111111111111111111111111111111'}],payer,context).TAG).toBe('Error');
 expect(V1.fromWireBytes(new Uint8Array([128]),context).TAG).toBe('Error');
});
