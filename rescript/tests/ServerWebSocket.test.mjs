import {test,expect} from 'bun:test';
import * as Server from '../src/ServerApi.res.mjs';
import * as Client from '../src/Client.res.mjs';
import * as Ws from '../src/ws/Ws.res.mjs';
test('server reconnect reads current native token in handshake',async()=>{
 const cookies=[];
 const server=Bun.serve({hostname:'127.0.0.1',port:0,fetch(request,server){cookies.push(request.headers.get('cookie'));if(server.upgrade(request))return;return new Response('upgrade required',{status:400})},websocket:{message(){}}});
 const c=Client.make(undefined,undefined,`ws://127.0.0.1:${server.port}`,undefined,undefined,undefined,undefined,undefined,undefined,undefined);
 Server.setAuthToken(c,'first');let connection;
 try{
  let connected;let ready=new Promise(r=>connected=r);connection=Server.connect(c,()=>{},()=>connected(),()=>{});await ready;
  Server.setAuthToken(c,'second');ready=new Promise(r=>connected=r);Ws.restartConnection(connection);await ready;
  expect(cookies).toEqual(['lightcone-token=first','lightcone-token=second']);
 }finally{if(connection)Ws.stop(connection);server.stop(true)}
});
