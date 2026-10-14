import {test,expect,afterEach} from 'bun:test';
import {ed25519} from '@noble/curves/ed25519';
import * as kit from '@solana/kit';
import * as Client from '../src/Client.res.mjs';
import * as Sol from '../src/domain/position/Position__Sol.res.mjs';
import * as State from '../src/domain/position/Position__WalletState.res.mjs';
import * as Ix from '../src/program/SolInstructions.res.mjs';
import * as V1 from '../src/program/V1Transaction.res.mjs';
const original=fetch;afterEach(()=>{globalThis.fetch=original});
const ok=r=>{expect(r.TAG,JSON.stringify(r,(_,v)=>typeof v==='bigint'?String(v):v)).toBe('Ok');return r._0};
const key=n=>kit.getAddressDecoder().decode(new Uint8Array(32).fill(n));
const seed=new Uint8Array(32).fill(1),secret=new Uint8Array([...seed,...ed25519.getPublicKey(seed)]),wallet=kit.getAddressDecoder().decode(secret.slice(32));
const resources={computeUnitLimit:200000,loadedAccountsDataSizeLimit:1048576,priorityFeeLamports:0n};
const tokenProgram='TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';
const canonical=await Ix.canonical(wallet);
const sol=n=>`${n/1000000000n}.${String(n%1000000000n).padStart(9,'0')}`;
async function setup(native=10000000n,wsol=0n,external=false){
 const c=Client.make(undefined,undefined,undefined,'https://rpc.test',undefined,undefined,undefined,resources,undefined,undefined);
 if(external)Client.useExternalSigner(c,wallet,async()=>new Uint8Array(),async()=>new Uint8Array());else await Client.useNativeSigner(c,secret);
 c.credentials={userId:'user',walletAddress:wallet,expiresAt:Date.now()/1000+60};
 const state=State.make();State.applyRestSnapshot(state,wallet,{contextSlot:42n,nativeSolBalance:sol(native),balances:wsol?{[Ix.mint]:{mint:Ix.mint,idle:sol(wsol),name:'Solana',symbol:'SOL'}}:{}});
 return {c,state};
}
function account(amount,donation=0n){
 const data=Buffer.alloc(165);data.set(kit.getAddressEncoder().encode(Ix.mint));data.set(kit.getAddressEncoder().encode(wallet),32);data.writeBigUInt64LE(amount,64);data[108]=1;data.writeUInt32LE(1,109);data.writeBigUInt64LE(2039280n,113);
 return {owner:tokenProgram,lamports:Number(amount+2039280n+donation),data:[data.toString('base64'),'base64']};
}
function rpc(live,fee=5000){let calls=[];globalThis.fetch=async(_,init)=>{const {method,params}=JSON.parse(init.body);calls.push({method,params});let result;
 if(method==='getAccountInfo')result={value:params[0]===canonical?live:null};
 else if(method==='getMinimumBalanceForRentExemption')result=2039280;
 else if(method==='getLatestBlockhash')result={value:{blockhash:key(7),lastValidBlockHeight:100}};
 else if(method==='getFeeForMessage')result={value:typeof fee==='function'?fee():fee};
 else throw Error(method);return new Response(JSON.stringify({result}));};return calls}
const decoded=plan=>kit.getCompiledTransactionMessageDecoder().decode(V1.messageBytes(plan.transaction));
const instructions=plan=>{const m=decoded(plan);return m.instructionHeaders.map((h,i)=>({program:m.staticAccounts[h.programAccountIndex],data:m.instructionPayloads[i].instructionData,accounts:m.instructionPayloads[i].instructionAccountIndices.map(j=>m.staticAccounts[j])}))};
test('temporary withdrawal seed and derived account match independent Rust vector',()=>{
 const seed=Ix.nativeWithdrawSeed(key(0),key(1),key(2),0x0102030405060708n,7);
 expect(seed).toBe('4dce744c636478f024df5aefd987f933');expect(Ix.temporaryAccount(key(1),seed)).toBe('71S4MLz9scZhY8BomAjfTkVn6HhFo8yFU7G6tSLto5g6');
});
test('wrap creates strictly, transfers exactly and projects rent and fee separately',async()=>{
 const {c,state}=await setup();rpc(null);const plan=ok(await Sol.planWrap(c,1000000n,state));
 expect(instructions(plan).map(i=>[i.program,i.data[0]])).toEqual([[ 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',0],['11111111111111111111111111111111',2],[tokenProgram,17]]);
 expect(plan.expectedDelta).toEqual({nativeLamports:-3044280n,canonicalWsolLamports:1000000n});expect(plan.availability.reserveLamports).toBe(3500000n);
 expect(State.nativeSolBalance(state)).toBe('0.010000000');
});
test('native conversions reject external signers and unsynchronized donated lamports',async()=>{
 let {c,state}=await setup(10000000n,1000000n,true);let calls=rpc(account(1000000n));expect((await Sol.planWrap(c,1n,state)).TAG).toBe('Error');expect(calls.length).toBe(0);
 ({c,state}=await setup(10000000n,1000000n));rpc(account(1000000n,1n));expect((await Sol.planWrap(c,1n,state)).TAG).toBe('Error');
 const plan=ok(await Sol.planUnwrapAll(c,state));expect(plan.expectedDelta).toEqual({nativeLamports:3034281n,canonicalWsolLamports:-1000000n});expect(plan.availability.reserveLamports).toBe(5000n);expect(instructions(plan).map(i=>i.data[0])).toEqual([9]);
});
test('ordinary browser split spends WSOL first and never closes canonical account',async()=>{
 const {c,state}=await setup(10000000n,1000000n,true);rpc(account(1000000n));
 const plan=ok(await Sol.planSplit(c,key(9),2,1500000n,state));const ixs=instructions(plan);
 expect(ixs.map(i=>i.data[0])).toEqual([2,17,3]);expect(ixs[0].data).toEqual(new Uint8Array([2,0,0,0,32,161,7,0,0,0,0,0]));
 expect(plan.expectedDelta).toEqual({nativeLamports:-505000n,canonicalWsolLamports:-1000000n});
});
test('merge and redeem retain canonical proceeds; unavailable live fees fail closed',async()=>{
 const {c,state}=await setup();rpc(null);const merge=ok(await Sol.planMerge(c,key(9),2,1500000n,state));expect(instructions(merge).map(i=>i.data[0])).toEqual([1,4]);expect(merge.expectedDelta).toEqual({nativeLamports:-2044280n,canonicalWsolLamports:1500000n});
 const redeem=ok(await Sol.planRedeem(c,key(9),2,1,1500000n,state));expect(instructions(redeem).map(i=>i.data[0])).toEqual([1,8]);
 rpc(null,null);expect((await Sol.planMerge(c,key(9),2,1n,state)).TAG).toBe('Error');
});
test('native withdrawal is direct when funded or atomically converts only the shortfall',async()=>{
 let {c,state}=await setup();rpc(null);const direct=ok(await Sol.planNativeWithdrawal(c,key(2),1000000n,state));expect(instructions(direct).map(i=>i.data[0])).toEqual([2]);expect(direct.expectedDelta.nativeLamports).toBe(-1005000n);
 ({c,state}=await setup(3000000n,10000000n,true));rpc(account(10000000n));const temporary=ok(await Sol.planNativeWithdrawal(c,key(2),5000000n,state));const ixs=instructions(temporary);
 expect(ixs.map(i=>i.data[0])).toEqual([3,18,3,9,2]);expect(ixs[3].accounts[0]).not.toBe(canonical);expect(ixs[2].accounts[0]).toBe(canonical);
 expect(temporary.expectedDelta).toEqual({nativeLamports:-960720n,canonicalWsolLamports:-4044280n});
});
test('canonical decoding rejects frozen/wrong-owner/truncated accounts and mismatched auth fails before RPC',async()=>{
 const {c,state}=await setup(10000000n,1000000n);let live=account(1000000n);live.owner=key(4);rpc(live);expect((await Sol.planSplit(c,key(9),2,1n,state)).TAG).toBe('Error');
 live=account(1000000n);const data=Buffer.from(live.data[0],'base64');data[108]=2;live.data[0]=data.toString('base64');rpc(live);expect((await Sol.canonicalAccount(c,wallet)).TAG).toBe('Error');
 c.credentials.walletAddress=key(2);const calls=rpc(null);expect((await Sol.planSplit(c,key(9),2,1n,state)).TAG).toBe('Error');expect(calls.length).toBe(0);
});
