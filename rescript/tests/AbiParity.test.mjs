import {test,expect} from 'bun:test';
import * as kit from '@solana/kit';
import {ed25519} from '@noble/curves/ed25519';
import * as Ix from '../src/program/Instructions.res.mjs';
import * as Accounts from '../src/program/Accounts.res.mjs';
const key=n=>kit.getAddressDecoder().decode(new Uint8Array(32).fill(n));
const wallet=kit.getAddressDecoder().decode(ed25519.getPublicKey(new Uint8Array(32).fill(1)));
const program=key(99),market=key(9),base=key(8),quote=key(7),baseDeposit=key(6),quoteDeposit=key(5);
const ok=r=>{expect(r.TAG).toBe('Ok');return r._0};
const address=kit.getAddressEncoder();
const pda=async seeds=>(await kit.getProgramDerivedAddress({programAddress:program,seeds}))[0];
const text=s=>new TextEncoder().encode(s);
const trailer=async ix=>expect(ix.accounts.slice(-2)).toEqual([{address:await pda([text('__event_authority')]),role:0},{address:program,role:0}]);
const signed=(side,salt=1n)=>({order:{maker:wallet,market,baseMint:base,quoteMint:quote,side,nonce:3n,salt,amountIn:10n,amountOut:10n,expiration:0n},signature:new Uint8Array(64)});
test('matching encodes eleven makers and bits 7, 8, and 15 in a u16 mask',async()=>{
 const makers=Array.from({length:11},(_,i)=>({order:signed(1,BigInt(i+1)),makerFillAmount:1n,takerFillAmount:1n,isFullFill:i===7||i===8}));
 const ix=ok(await Ix.matchOrdersMulti(program,wallet,market,base,quote,baseDeposit,quoteDeposit,wallet,signed(0),true,makers));
 expect(ix.data.length).toBe(105+117*11);expect([...ix.data.slice(102,105)]).toEqual([11,0x80,0x81]);
 expect(ix.accounts.slice(4,6)).toEqual([{address:await pda([text('global_deposit'),address.encode(quoteDeposit)]),role:0},{address:await pda([text('global_deposit'),address.encode(baseDeposit)]),role:0}]);
 expect(ix.accounts[7].role).toBe(0);await trailer(ix);
 expect((await Ix.matchOrdersMulti(program,wallet,market,base,quote,baseDeposit,quoteDeposit,wallet,signed(0),true,[...makers,makers[0]])).TAG).toBe('Error');
});
test('permissionless initialization omits ALT, preserves beneficiary role and limits distinct groups',async()=>{
 const ix=ok(await Ix.initPositionTokens(program,wallet,wallet,market,[baseDeposit,quoteDeposit],2));expect([...ix.data]).toEqual([19,2]);expect(ix.accounts[0]).toEqual({address:wallet,role:3});expect(ix.accounts[1]).toEqual({address:wallet,role:0});await trailer(ix);
 expect((await Ix.initPositionTokens(program,wallet,wallet,market,[baseDeposit,baseDeposit],2)).TAG).toBe('Error');
 expect((await Ix.initPositionTokens(program,wallet,key(0),market,[baseDeposit],2)).TAG).toBe('Error');
});
test('176-byte orderbook and strict 47-byte GDT reject obsolete and malformed layouts',()=>{
 const book=new Uint8Array(176);book.set([0x2b,0x22,0x19,0x71,0xc3,0x45,0x48,0x07]);book.set(address.encode(baseDeposit),104);book.set(address.encode(quoteDeposit),136);book[168]=1;book[169]=5;book[170]=255;
 const decoded=ok(Accounts.Orderbook.decode(book));expect(decoded.depositMintA).toBe(baseDeposit);expect(decoded.depositMintB).toBe(quoteDeposit);expect(Accounts.Orderbook.baseDepositMint(decoded)).toBe(quoteDeposit);
 for(const bytes of [book.slice(0,144),new Uint8Array([...book,0])])expect(Accounts.Orderbook.decode(bytes).TAG).toBe('Error');book[169]=6;expect(Accounts.Orderbook.decode(book).TAG).toBe('Error');
 const gdt=new Uint8Array(47);gdt.set([0x25,0xbe,0xa1,0xe8,0x7b,0x92,0x2a,0x57]);gdt[43]=1;ok(Accounts.GlobalDepositToken.decode(gdt));gdt[43]=2;expect(Accounts.GlobalDepositToken.decode(gdt).TAG).toBe('Error');
});
