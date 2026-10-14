import {expect,test} from 'bun:test';
import * as Ws from '../src/ws/Ws.res.mjs';
const sub=(TAG,_0)=>({TAG,_0});
class Socket {
 readyState=0;sent=[];closed=false;
 send(value){if(this.readyState!==1)throw Error('not open');this.sent.push(JSON.parse(value));}
 close(){this.closed=true;this.readyState=3;this.onclose?.({code:1000,reason:'closed'});}
 open(){this.readyState=1;this.onopen?.();}
}
function setup(){
 const sockets=[],messages=[],errors=[];
 const connection=Ws.connect('wss://example',()=>{const socket=new Socket();sockets.push(socket);return socket;},m=>messages.push(m),undefined,undefined,e=>errors.push(e),{reconnect:true,maxReconnectAttempts:3,baseReconnectDelayMs:1,pingIntervalMs:999999,pongTimeoutMs:999999});
 return {connection,sockets,messages,errors};
}
test('connecting sends flush once; stop drops queue and restart fences old callbacks',()=>{
 const {connection,sockets,messages}=setup();
 try {
  Ws.subscribe(connection,sub('User','wallet'));Ws.ping(connection);
  sockets[0].open();expect(sockets[0].sent.map(m=>m.method)).toEqual(['subscribe','ping']);
  Ws.restartConnection(connection);const old=sockets[0];Ws.ping(connection);Ws.stop(connection);
  expect(Ws.isStopped(connection)).toBe(true);
  Ws.restartConnection(connection);old.onmessage({data:'{"type":"pong"}'});old.onopen();
  sockets.at(-1).open();expect(messages).toEqual([]);
  expect(sockets.at(-1).sent.map(m=>m.method)).toEqual(['subscribe']);
 }finally{Ws.stop(connection);}
});
test('logout drops both private subscription intents while retaining books; AUTH_REQUIRED stays a message',()=>{
 const {connection,sockets,messages}=setup();
 try{
  Ws.subscribe(connection,sub('User','wallet'));Ws.subscribe(connection,sub('WalletDepositBalances','wallet'));
  Ws.subscribe(connection,sub('Books',{orderbookIds:['book']}));Ws.clearAuthedSubscriptions(connection);
  sockets[0].open();expect(sockets[0].sent.map(m=>m.params.type)).toEqual(['book_update']);
  sockets[0].onmessage({data:JSON.stringify({type:'error',data:{error:'login needed',code:'AUTH_REQUIRED'}})});
  expect(messages[0].kind.TAG).toBe('ErrorFrame');expect(Ws.isConnected(connection)).toBe(true);
 }finally{Ws.stop(connection);}
});
test('restart during connecting replaces handshake and deduplicates generic errors',()=>{
 const {connection,sockets,errors}=setup();
 try{
  Ws.restartConnection(connection);expect(sockets[0].closed).toBe(true);
  sockets[0].onerror({});expect(errors).toEqual([]);
  sockets[1].onerror({});sockets[1].onerror({});expect(errors.length).toBe(1);
  sockets[1].open();expect(Ws.isConnected(connection)).toBe(true);
 }finally{Ws.stop(connection);}
});
