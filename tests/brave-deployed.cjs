// Fresh Brave with CDP on :19222; navigate to the real HTTPS deployment.
async function main() {
 let pages;for(let i=0;i<80;i++){try{pages=await(await fetch('http://127.0.0.1:19222/json')).json();if(pages.length)break}catch{}await new Promise(r=>setTimeout(r,250));}
 if(!pages?.length)throw Error('No Brave target');
 const ws=new WebSocket(pages[0].webSocketDebuggerUrl);await new Promise((ok,fail)=>{ws.onopen=ok;ws.onerror=fail});let id=0;const pending=new Map();ws.onmessage=e=>{const m=JSON.parse(e.data);if(pending.has(m.id)){pending.get(m.id)(m);pending.delete(m.id)}};
 const call=(method,params={})=>new Promise(resolve=>{const n=++id;pending.set(n,resolve);ws.send(JSON.stringify({id:n,method,params}));});
 const evaluate=async expression=>{const r=await call('Runtime.evaluate',{awaitPromise:true,returnByValue:true,expression});if(r.result?.exceptionDetails||!r.result?.result?.value)throw Error(JSON.stringify(r));return r.result.result.value;};
 const delay=ms=>new Promise(r=>setTimeout(r,ms));
 await call('Page.enable');await call('Runtime.enable');await call('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:3,mobile:true});
 await call('Page.navigate',{url:'https://sudoku.scturtle.me/'});await delay(4000);
 let checks=await evaluate(`(async()=>{const w=()=>new Promise(r=>setTimeout(r,200));const secure=location.protocol==='https:'&&location.hostname==='sudoku.scturtle.me'&&window.isSecureContext;const home=!!document.querySelector('.home-nav')&&!!document.querySelector('.learning-link');document.querySelector('.basic-course').click();await w();const game=!!document.querySelector('.game-header')&&document.querySelectorAll('[role=gridcell]').length===81;const saved=!!localStorage.getItem('sudoku402.game.v1');document.querySelector('.back-button').click();await w();return {secure,home,game,saved,learnLink:document.querySelector('.learning-link').getAttribute('href')==='help/index.html'}})()`);
 await call('Runtime.evaluate',{expression:"document.querySelector('.learning-link').click()"});await delay(1000);
 Object.assign(checks,await evaluate(`(()=>({catalog:location.pathname==='/help/index.html'&&document.querySelectorAll('li a').length===24,white:getComputedStyle(document.body).backgroundColor==='rgb(255, 255, 255)',savedAfterNavigation:!!localStorage.getItem('sudoku402.game.v1')}))()`));
 console.log(JSON.stringify(checks));ws.close();if(Object.values(checks).some(v=>!v))process.exitCode=1;
}
main().catch(e=>{console.error(e);process.exitCode=1});
