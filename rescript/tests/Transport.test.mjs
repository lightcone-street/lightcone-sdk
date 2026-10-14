import { afterEach, expect, test } from 'bun:test';
import * as Http from '../src/Http.res.mjs';
import * as Errors from '../src/SdkError.res.mjs';
const originalFetch = globalThis.fetch;
afterEach(() => { globalThis.fetch = originalFetch; delete globalThis.window; delete globalThis.WorkerGlobalScope; });
const ok = value => ({TAG:'Ok',_0:value});
const response = (status, body, headers) => new Response(JSON.stringify(body), {status,headers});
const success = value => response(200,{status:'success',body:value});
const get = (http, cookie, retry) => Http.get(http,'/api/test',undefined,retry,cookie,undefined,ok);
const post = (http, path='/api/orders/submit', retry) => Http.post(http,path,{amount:9007199254740993n},retry,undefined,undefined,ok);

test('concurrent 401 requests share restoration and replay once with stable IDs', async () => {
  const http = Http.make('https://api.test');
  let restorations = 0, valid = false;
  Http.setCredentialRestorer(http, async () => {
    restorations++; await new Promise(resolve => setTimeout(resolve,10)); valid = true; return true;
  });
  const calls = [];
  globalThis.fetch = async (url,init) => {calls.push(init.headers['x-request-id']); return valid ? success('done') : response(401,{});};
  const results = await Promise.all([get(http),get(http)]);
  expect(results).toEqual([ok('done'),ok('done')]);
  expect(restorations).toBe(1);
  expect(calls.length).toBe(4);
  expect(new Set(calls).size).toBe(2);
});
test('mutations heal credentials without replay; nonce/login/logout and explicit cookies never restore', async () => {
  const http = Http.make('https://api.test'); let calls=0, restores=0;
  Http.setCredentialRestorer(http, async () => {restores++; return true;});
  globalThis.fetch = async () => {calls++; return response(401,{});};
  expect(Errors.isUnauthorized((await post(http))._0)).toBe(true);
  expect(calls).toBe(1); expect(restores).toBe(1);
  await post(http,'/api/auth/logout'); await post(http,'/api/auth/login_or_register_with_message');
  await Http.get(http,'/api/auth/nonce',undefined,'NoRetry',undefined,undefined,ok);
  await get(http,'privy-token=ssr-user');
  expect(restores).toBe(1);
});
test('second 401 and 403 never loop restoration', async () => {
  const http = Http.make('https://api.test'); let restores=0,calls=0;
  Http.setCredentialRestorer(http,async()=>{restores++;return true;});
  globalThis.fetch=async()=>{calls++;return response(401,{});};
  await get(http); expect(calls).toBe(2); expect(restores).toBe(1);
  globalThis.fetch=async()=>response(403,{}); await get(http); expect(restores).toBe(1);
});
test('Privy outage preserves structured classification with malformed optional metadata and does not retry',async()=>{
  const http=Http.make('https://api.test'); let calls=0;
  globalThis.fetch=async()=>{calls++; return response(503,{status:'error',error_details:{error_code:'PRIVY_VERIFICATION_UNAVAILABLE',reason:'Try again',existing_method:42,rejection_code:{bad:true}}},{'retry-after-ms':'1.2'});};
  const result=await get(http);
  expect(calls).toBe(1);expect(Errors.isPrivyVerificationUnavailable(result._0)).toBe(true);
  expect(result._0._0.retryAfterMs).toBe(2);expect(result._0._0.requestId).toBeString();
});
test('cookie capture is limited to successful own-session responses',async()=>{
  const http=Http.make('https://api.test'); Http.setAuthToken(http,'owner');
  globalThis.fetch=async()=>response(200,{status:'success',body:1},{'set-cookie':'lightcone-token=other; HttpOnly'});
  await get(http,'privy-token=other'); expect(Http.authToken(http)).toBe('owner');
  await get(http); expect(Http.authToken(http)).toBe('other');
  globalThis.fetch=async()=>response(401,{}, {'set-cookie':'lightcone-token=attacker'});
  await get(http);expect(Http.authToken(http)).toBe('other');
});
test('retries preserve exact body bytes and request ID',async()=>{
  const http=Http.make('https://api.test');const calls=[];
  globalThis.fetch=async(url,init)=>{calls.push(init);return calls.length===1 ? response(503,{}) : success(1);};
  const retry={TAG:'Custom',_0:{maxRetries:1,initialDelayMs:0,maxDelayMs:0,backoffFactor:1,jitter:false,retryableStatuses:[503]}};
  expect(await post(http,'/api/test',retry)).toEqual(ok(1));
  expect(calls[0].body).toBe('{"amount":9007199254740993}');
  expect(calls[1].body).toBe(calls[0].body);
  expect(calls[1].headers['x-request-id']).toBe(calls[0].headers['x-request-id']);
});
test('server keys only reach approved routes and secure origins; browser and workers reject them',async()=>{
  const http=Http.make('https://api.test');expect(Http.setServerApiKey(http,'secret')).toEqual(ok(undefined));
  const calls=[];globalThis.fetch=async(url,init)=>{calls.push(init);return success(1);};
  await post(http);await post(http,'/api/auth/register-privy');await get(http);
  expect(calls.map(call=>call.headers['x-lightcone-api-key'])).toEqual(['secret',undefined,undefined]);
  expect(calls.every(call=>call.redirect==='error')).toBe(true);
  expect(Http.setServerApiKey(Http.make('http://remote.test'),'secret').TAG).toBe('Error');
  expect(Http.setServerApiKey(Http.make('http://127.0.0.1:4000'),'secret').TAG).toBe('Ok');
  globalThis.WorkerGlobalScope=class {};expect(Http.setServerApiKey(http,'secret').TAG).toBe('Error');
});
test('Retry-After accepts only valid HTTP dates and millisecond headers have precedence',async()=>{
 const http=Http.make('https://api.test');
 for(const [headers,expected] of [
  [{'retry-after':'Sun, 06 Nov 1994 08:49:37 GMT'},0],
  [{'retry-after':'Sunday, 06-Nov-94 08:49:37 GMT'},0],
  [{'retry-after':'Sun Nov  6 08:49:37 1994'},0],
  [{'retry-after':'Mon, 06 Nov 1994 08:49:37 GMT'},undefined],
  [{'retry-after':'November 6, 1994'},undefined],
  [{'retry-after-ms':'1.1','retry-after':'10'},2],
 ]){
  globalThis.fetch=async()=>response(429,{},headers);
  const result=await get(http,undefined,'NoRetry');expect(result._0._0.retryAfterMs).toBe(expected);
 }
});
