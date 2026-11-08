// Requires fresh Brave CDP :19222 and the application on HTTP :18765.
async function main() {
  let pages; for(let i=0;i<80;i++){try{pages=await(await fetch('http://127.0.0.1:19222/json')).json();if(pages.length)break}catch{}await new Promise(r=>setTimeout(r,250))}
  if(!pages?.length)throw Error('No Brave target'); const ws=new WebSocket(pages[0].webSocketDebuggerUrl);
  await new Promise((ok,fail)=>{ws.onopen=ok;ws.onerror=fail});let id=0;const callbacks=new Map();
  ws.onmessage=e=>{const m=JSON.parse(e.data);if(callbacks.has(m.id)){callbacks.get(m.id)(m);callbacks.delete(m.id)}};
  const call=(method,params={})=>new Promise(resolve=>{const n=++id;callbacks.set(n,resolve);ws.send(JSON.stringify({id:n,method,params}))});
  await call('Page.enable');await call('Runtime.enable');await call('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:3,mobile:true});
  await call('Page.navigate',{url:'http://127.0.0.1:18765/'});await new Promise(r=>setTimeout(r,3500));
  async function fixture(auto,first,second) {
    const saved={version:1,puzzle:'s401-8988',values:Array(81).fill(0),candidates:Array(81).fill(0),auto_candidates:auto};
    saved.candidates[5]=1<<first;saved.candidates[14]=(1<<first)|(1<<second);
    await call('Runtime.evaluate',{expression:`localStorage.setItem('sudoku402.game.v1',${JSON.stringify(JSON.stringify(saved))})`});
    await call('Page.reload');await new Promise(r=>setTimeout(r,2500));
    await call('Runtime.evaluate',{expression:`document.querySelector('.continue-row').click()`});await new Promise(r=>setTimeout(r,200));
  }
  async function evaluate(expression){const r=await call('Runtime.evaluate',{awaitPromise:true,returnByValue:true,expression});if(!r.result?.result?.value)throw Error(JSON.stringify(r));return r.result.result.value}
  const checks={};
  await fixture(true,2,7);
  Object.assign(checks,await evaluate(`(async()=>{const w=()=>new Promise(r=>setTimeout(r,100));const c=()=>document.querySelectorAll('[role=gridcell]');const b=[...document.querySelectorAll('.controls button')];const position=b[1].textContent==='快进';b[1].click();await w();const chain=c()[5].textContent.trim()==='2'&&c()[14].textContent.trim()==='7';b[0].click();await w();const undo=c()[5].getAttribute('aria-label').includes('空')&&c()[14].getAttribute('aria-label').includes('空')&&c()[14].querySelectorAll('.candidates span')[1].textContent==='2';return {position,chain,undo}})()`));
  await fixture(false,2,7);
  Object.assign(checks,await evaluate(`(async()=>{const w=()=>new Promise(r=>setTimeout(r,100));document.querySelector('.fast-forward').click();await w();const c=document.querySelectorAll('[role=gridcell]');return {manual:c[5].textContent.trim()==='2'&&c[14].getAttribute('aria-label').includes('空')&&c[14].querySelectorAll('.candidates span')[1].textContent==='2'}})()`));
  await fixture(true,1,7);
  Object.assign(checks,await evaluate(`(async()=>{const w=()=>new Promise(r=>setTimeout(r,100));document.querySelector('.fast-forward').click();await w();const c=document.querySelectorAll('[role=gridcell]');const stopped=c[5].textContent.trim()==='1'&&c[5].classList.contains('conflict')&&c[14].getAttribute('aria-label').includes('空')&&c[14].querySelectorAll('.candidates span')[0].textContent==='1';const message=document.querySelector('.message').textContent.includes('快进已停止');const saved=JSON.parse(localStorage.getItem('sudoku402.game.v1'));document.querySelector('.fast-forward').click();await w();const existingStopped=JSON.stringify(saved)===localStorage.getItem('sudoku402.game.v1');document.querySelector('.controls button').click();await w();return {stopped,message,existingStopped,undoWrong:c[5].getAttribute('aria-label').includes('空')}})()`));
  const clues='369510748548390216007680305456209071291078603083165024030900000815726039974853062';
  const answer='369512748548397216127684395456239871291478653783165924632941587815726439974853162';
  const final={version:1,puzzle:'s401-8988',values:[...clues].map((v,i)=>v==='0'&&i!==5?Number(answer[i]):0),candidates:Array(81).fill(0),auto_candidates:true};final.candidates[5]=1<<2;
  await call('Runtime.evaluate',{expression:`localStorage.setItem('sudoku402.game.v1',${JSON.stringify(JSON.stringify(final))})`});await call('Page.reload');await new Promise(r=>setTimeout(r,2500));
  Object.assign(checks,await evaluate(`(async()=>{const w=()=>new Promise(r=>setTimeout(r,100));document.querySelector('.continue-row').click();await w();document.querySelector('.fast-forward').click();await w();return {silentCompletion:!document.querySelector('.game-screen').hidden&&!document.querySelector('.message').textContent.trim()&&!document.querySelector('[role=alertdialog]')&&document.querySelectorAll('[role=gridcell]')[5].textContent.trim()==='2'}})()`));
  console.log(JSON.stringify(checks));ws.close();if(Object.values(checks).some(v=>!v))process.exitCode=1;
}
main().catch(e=>{console.error(e);process.exitCode=1});
