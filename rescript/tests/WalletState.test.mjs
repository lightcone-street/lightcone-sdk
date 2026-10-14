import {expect,test} from 'bun:test';
import * as State from '../src/domain/position/Position__WalletState.res.mjs';
import * as Raw from '../src/domain/position/Position__Raw.res.mjs';
import * as Messages from '../src/ws/Messages.res.mjs';
const row=(mint,idle)=>({mint,idle,name:mint,symbol:mint});
const snapshot=(contextSlot,nativeSolBalance,balances={})=>({contextSlot,nativeSolBalance,balances});
const event=(TAG,_0)=>({TAG,_0});
test('complete snapshots replace newer component slots; floors apply only to snapshots',()=>{
 const state=State.make();
 State.applyRestSnapshot(state,'wallet',snapshot(10n,'1.000000000',{old:row('old','1')}));
 State.applyEvent(state,event('NativeSolBalanceUpdate',{walletAddress:'wallet',contextSlot:100n,nativeSolBalance:'9.000000000'}));
 expect(State.applyRestSnapshot(state,'wallet',snapshot(20n,'2.000000000'))).toBe('Applied');
 expect(State.balances(state)).toEqual({});expect(State.contextSlot(state)).toBe(20n);
 expect(State.applyRestSnapshot(state,'wallet',snapshot(19n,'3.000000000'),20n)).toBe('Ignored');
 expect(State.applyRestSnapshot(state,'wallet',snapshot(20n,'3.000000000'),20n)).toBe('Applied');
 expect(State.applyEvent(state,event('BalanceUpdate',{walletAddress:'wallet',contextSlot:1n,balance:row('mint','2')}),1000n)).toBe('Applied');
 expect(State.contextSlot(state)).toBe(1n);
});
test('uninitialized and wrong-wallet updates are ignored; negative values and status do not mutate',()=>{
 const state=State.make(); const update=(idle,walletAddress='wallet')=>event('BalanceUpdate',{walletAddress,contextSlot:99n,balance:row('mint',idle)});
 expect(State.applyEvent(state,update('1'))).toBe('Ignored');
 State.applyRestSnapshot(state,'wallet',snapshot(1n,'0.000000001'));
 expect(State.applyEvent(state,update('1','other'))).toBe('Ignored');
 expect(State.applyEvent(state,update('-1'))).toBe('Rejected');expect(State.contextSlot(state)).toBe(1n);
 expect(State.applyEvent(state,update('2'))).toBe('Applied');expect(State.balances(state).mint.idle).toBe('2');
 expect(State.applyEvent(state,update('0'))).toBe('Applied');expect(State.balances(state)).toEqual({});
 expect(State.applyEvent(state,event('Status',{walletAddress:'wallet',status:'reconnecting',code:'retry'}))).toBe('Ignored');
});
test('SOL display adds exact arbitrary-width amounts and transaction breakdown rejects overflow',()=>{
 const state=State.make();const wsol=State.wrappedSolMint;
 State.applyRestSnapshot(state,'wallet',snapshot(1n,'18446744073.709551615',{[wsol]:row(wsol,'0.000000001')}));
 expect(State.combinedSolBalance(state)).toEqual({TAG:'Ok',_0:'18446744073.709551616'});
 State.applyRestSnapshot(state,'wallet',snapshot(1n,'18446744073.709551616'));
 expect(State.solBalanceBreakdown(state).TAG).toBe('Error');
});
test('required snapshot fields and canonical native SOL survive no permissive defaults',()=>{
 const decode=Raw.DepositTokenBalancesSnapshot.t_decode;
 const base={context_slot:1,balances:{},native_sol_balance:'0.000000001'};
 expect(decode(base).TAG).toBe('Ok');
 for(const value of ['1','01.000000000','-1.000000000','1e0','1.00000000',1,null])expect(decode({...base,native_sol_balance:value}).TAG).toBe('Error');
 expect(decode({context_slot:1,balances:{}}).TAG).toBe('Error');
 expect(decode({...base,balances:null}).TAG).toBe('Error');
 const message={type:'wallet_deposit_balances',data:{...base,event_type:'wallet_deposit_balance_snapshot',wallet_address:'wallet',balances:{mint:null}}};
 expect(Messages.decodeMessage(message).TAG).toBe('Error');
 message.data.balances={mint:{mint:'mint',idle:'1',name:'M',symbol:'M',icon_url_low:null}};
 expect(Messages.decodeMessage(message).TAG).toBe('Ok');
});
