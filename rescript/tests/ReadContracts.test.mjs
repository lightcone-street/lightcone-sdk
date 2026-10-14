import {test,expect,afterEach} from 'bun:test';
import * as Raw from '../src/auth/Auth__Raw.res.mjs';
import * as Auth from '../src/auth/Auth__Client.res.mjs';
import * as Model from '../src/auth/Auth__Model.res.mjs';
import * as Client from '../src/Client.res.mjs';
import * as MarketRaw from '../src/domain/market/Market__Raw.res.mjs';
import * as Markets from '../src/domain/market/Market__Client.res.mjs';
import * as Errors from '../src/SdkError.res.mjs';
import * as Exact from '../src/ExactJson.res.mjs';
import * as Fmt from '../src/fmt/Fmt__Decimal.res.mjs';
import Decimal from 'decimal.js';
import {marketFixture} from './market-fixture.mjs';
const original=fetch;afterEach(()=>{globalThis.fetch=original});
const ok=r=>{expect(r.TAG).toBe('Ok');return r._0};
const session=()=>({user:{user_id:'user',identity:{type:'email',account:{email:'email@example.com'},privy:{id:'id',wallet:{privy_id:'walletid',chain:'solana',address:'trading-wallet'}}},linked_identities:[{type:'wallet',address:'linked-wallet',chain:'solana'}],max_slippage_preference:'0.000000000000000001',telegram_invite_url:'https://t.me/invite'},expires_at:Date.now()/1000+60,auth_method:'privy',is_beta:true});
const response=(status,body)=>new Response(JSON.stringify(body),{status});
const success=body=>response(200,{status:'success',body});
const client=()=>Client.make();
test('email and linked identities select trading wallet and preserve exact preferences',()=>{
 const decoded=ok(Raw.decodeSession(session()));expect(Model.tradingWallet(decoded.user,decoded.authMethod)).toBe('trading-wallet');expect(decoded.user.linkedIdentities[0].address).toBe('linked-wallet');expect(decoded.user.maxSlippagePreference).toBe('0.000000000000000001');expect(decoded.user.telegramInviteUrl).toBe('https://t.me/invite');
});
test('Privy registration always installs synchronized credentials; SSR and outage preserve them',async()=>{
 const c=client();let body;globalThis.fetch=async(_,init)=>{body=JSON.parse(init.body);return success(session())};ok(await Auth.registerPrivy(c,{attemptedIdentity:{TAG:'Email',_0:'email@example.com'}}));expect(body).toEqual({attempted_identity:{type:'email',email:'email@example.com'}});expect(Auth.isAuthenticated(c)).toBe(true);
 const credentials=Auth.credentials(c);globalThis.fetch=async()=>success({...session(),user:{...session().user,user_id:'other'}});ok(await Auth.checkSession(c,'cookie=isolated'));expect(Auth.credentials(c)).toEqual(credentials);
 globalThis.fetch=async()=>response(503,{status:'error',error_details:{error_code:'PRIVY_VERIFICATION_UNAVAILABLE',reason:'later'}});expect((await Auth.checkSession(c)).TAG).toBe('Error');expect(Auth.credentials(c)).toEqual(credentials);
 globalThis.fetch=async()=>response(401,{});await Auth.checkSession(c);expect(Auth.isAuthenticated(c)).toBe(false);
});
test('logout failure clears local auth and preference rejects invalid values before transport',async()=>{
 const c=client();c.credentials={userId:'u',walletAddress:'w',expiresAt:Date.now()/1000+60};globalThis.fetch=async()=>response(400,{});expect((await Auth.logout(c)).TAG).toBe('Error');expect(Auth.credentials(c)).toBeUndefined();
 let calls=0;globalThis.fetch=async()=>{calls++;return success({max_slippage_preference:'1.25'})};for(const input of ['0','-1','10','1.00000000000000000000000000001','NaN'])expect((await Auth.updateMaxSlippagePreference(c,input)).TAG).toBe('Error');expect(calls).toBe(0);expect(ok(await Auth.updateMaxSlippagePreference(c,'1.25'))).toBe('1.25');
});
test('market decoding keeps signed fees, exact millisecond deadlines, optional artwork and validated counts',()=>{
 const fixture=marketFixture();delete fixture.description;delete fixture.banner_image_url_low;fixture.outcomes=[{index:0,name:'Yes',icon_url_medium:'image'},{index:1,name:'No'}];fixture.maker_fee_bps=-10;fixture.taker_fee_bps=null;fixture.resolution_by='9007199254740993';
 const convert=source=>MarketRaw.MarketResponse.toMarket(ok(MarketRaw.MarketResponse.t_decode(Errors.stripNulls(source))));
 const market=ok(convert(fixture));expect(market.description).toBeUndefined();expect(market.bannerImageUrlLow).toBeUndefined();expect(market.makerFeeBps).toBe(-10);expect(market.takerFeeBps).toBe(0);expect(market.resolutionBy).toBe(9007199254740993n);expect(market.outcomes[0].iconUrlHigh).toBe('image');expect(market.outcomes[1].iconUrlLow).toBeUndefined();
 const inconsistent=structuredClone(fixture);inconsistent.num_outcomes=3;expect(convert(inconsistent).TAG).toBe('Error');delete fixture.num_outcomes;expect(ok(convert(fixture)).numOutcomes).toBe(2);delete fixture.definition;expect(convert(fixture).TAG).toBe('Error');
});
test('favorites preserve exact cursor and encode untrusted path segments',async()=>{
 const c=client(),calls=[];globalThis.fetch=async(url,init)=>{calls.push({url,init});return success(init.method==='GET'?{market_pubkeys:[],next_cursor:'9007199254740993',has_more:true}:{market_pubkey:'a/b',favorited:init.method!=='DELETE'})};
 const page=ok(await Markets.favoriteMarkets(c,10,9007199254740993n,'cookie=ssr'));expect(page.nextCursor).toBe(9007199254740993n);ok(await Markets.addFavoriteMarket(c,'a/b'));ok(await Markets.removeFavoriteMarket(c,'a/b'));expect(calls[0].url).toContain('cursor=9007199254740993');expect(calls[1].url).toEndWith('/a%2Fb');expect(calls[2].init.method).toBe('DELETE');
});
test('exact display preserves arbitrary precision in compact and expanded forms',()=>{
 expect(Fmt.displayExact(new Decimal('9007199254740993.000000000123'))).toEqual({compact:'9,007,199,254,740,993.000000000123',expanded:'9,007,199,254,740,993.000000000123'});
 expect(Fmt.displayExact(new Decimal('0.000000123'))).toEqual({compact:'0.0₆123',expanded:'0.000000123'});
});
