import {test,expect,afterEach} from 'bun:test';
import * as Client from '../src/Client.res.mjs';
import * as Books from '../src/domain/orderbook/Orderbook__Client.res.mjs';
import * as Envelope from '../src/program/Envelope.res.mjs';
import * as Orders from '../src/domain/order/Order__Client.res.mjs';
import * as Exact from '../src/ExactJson.res.mjs';
const original=fetch;afterEach(()=>{globalThis.fetch=original});
const client=()=>Client.make();
const wallet='So11111111111111111111111111111111111111112';
const rules=id=>({orderbook_id:id,base_decimals:6,quote_decimals:6,price_decimals:6,trading_rules:{base_size_decimals:6,max_price_decimals:6,max_price_significant_figures:5,integer_prices_always_allowed:true,price_quantum:'0.000001',price_quantum_raw:'1',base_size_quantum:'0.000001',base_size_quantum_raw:'1'}});
const success=body=>new Response(JSON.stringify({status:'success',body}));
test('rule lookups coalesce, validate identity, and failed entries can be retried',async()=>{
 const c=client();let calls=0;globalThis.fetch=async()=>{calls++;await new Promise(r=>setTimeout(r,2));return success(rules('book'))};
 const [a,b]=await Promise.all([Books.decimals(c,'book'),Books.decimals(c,'book')]);expect(a.TAG).toBe('Ok');expect(b).toEqual(a);expect(calls).toBe(1);
 Books.invalidateDecimals(c,'book');globalThis.fetch=async()=>{calls++;return success(rules('wrong'))};expect((await Books.decimals(c,'book')).TAG).toBe('Error');expect((await Books.decimals(c,'book')).TAG).toBe('Error');expect(calls).toBe(3);
});
test('invalid precision, field widths, and stale wallet identity never prompt the signer',async()=>{
 const c=client();let prompts=0;Client.useExternalSigner(c,wallet,async()=>{prompts++;return new Uint8Array(64)},async()=>new Uint8Array());
 globalThis.fetch=async()=>success(rules('book'));
 for(const [price,size,nonce,salt] of [['0.123456','1',0n,1n],['1','0.0000001',0n,1n],['1','1',4294967296n,1n],['1','1',0n,9223372036854775808n]]){
  const result=await Envelope.submitLimitOrderSigned(c,wallet,wallet,wallet,0,price,size,'book',nonce,salt);expect(result.TAG).toBe('Error');
 }
 expect(prompts).toBe(0);
 Books.clearDecimalsCache(c);globalThis.fetch=async()=>{Client.clearSigningStrategy(c);return success(rules('book'))};expect((await Envelope.submitLimitOrderSigned(c,wallet,wallet,wallet,0,'1','1','book',0n,1n)).TAG).toBe('Error');expect(prompts).toBe(0);
});
test('direct submission preserves integer tokens above 2^53 and rejects invalid expiration before POST',async()=>{
 const c=client();let posted;globalThis.fetch=async(_,init)=>{if(init.method==='GET')return success(rules('book'));posted=init.body;return success({order_hash:'hash',status:'accepted',remaining:'1',filled:'0',fills:[]})};
 const request={maker:wallet,marketPubkey:wallet,baseToken:wallet,quoteToken:wallet,side:0,amountIn:9007199254740993n,amountOut:9007199254740993n,nonce:0n,salt:1n,expiration:0n,signatureHex:'00'.repeat(64),orderbookId:'book',timeInForce:'alo'};
 expect((await Orders.submit(c,request)).TAG).toBe('Ok');expect(posted).toContain('9007199254740993');expect(posted).not.toContain('"9007199254740993"');expect(Exact.parse(posted)._0.amount_in).toBe('9007199254740993');
 posted=undefined;expect((await Orders.submit(c,{...request,expiration:9223372036854775808n})).TAG).toBe('Error');expect(posted).toBeUndefined();
});
