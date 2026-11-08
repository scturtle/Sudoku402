// Fresh Brave profile, CDP :19222 and the built app served at :18765.
async function main() {
 let pages;for(let i=0;i<80;i++){try{pages=await(await fetch('http://127.0.0.1:19222/json')).json();if(pages.length)break}catch{}await new Promise(r=>setTimeout(r,250))}
 if(!pages?.length)throw Error('No Brave target');const ws=new WebSocket(pages[0].webSocketDebuggerUrl);await new Promise((ok,fail)=>{ws.onopen=ok;ws.onerror=fail});
 let id=0;const pending=new Map();ws.onmessage=e=>{const m=JSON.parse(e.data);if(pending.has(m.id)){pending.get(m.id)(m);pending.delete(m.id)}};
 const call=(method,params={})=>new Promise(resolve=>{const n=++id;pending.set(n,resolve);ws.send(JSON.stringify({id:n,method,params}))});
 await call('Page.enable');await call('Runtime.enable');await call('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:3,mobile:true});
 await call('Page.navigate',{url:'http://127.0.0.1:18765/'});await new Promise(r=>setTimeout(r,3500));
 const r=await call('Runtime.evaluate',{awaitPromise:true,returnByValue:true,expression:`(async()=>{
  const wait=()=>new Promise(r=>setTimeout(r,100));const cells=()=>[...document.querySelectorAll('[role=gridcell]')];
  const button=t=>[...document.querySelectorAll('.controls button')].find(b=>b.textContent===t);
  const key=n=>[...document.querySelectorAll('.numbers button')].find(b=>b.textContent===String(n));
  const answer='416327589352891467798456213534219876287563941169748325623185794871934652945672138';
  Math.random=()=>0;document.querySelector('.easy-course').click();await wait();
  const entered=!document.querySelector('.game-screen').hidden&&document.querySelector('.game-header h1').textContent==='简单'&&cells().filter(c=>c.getAttribute('aria-label').includes('空')).length===57;
  const firstSaved=JSON.parse(localStorage.getItem('sudoku402.game.v1')).puzzle==='s401-10310';
  cells()[0].click();await wait();key(1).click();await wait();const wrong=cells()[0].classList.contains('conflict');
  button('提示').click();await wait();const errorFirst=cells()[0].classList.contains('hint-blue')&&document.querySelector('.message').textContent.includes('检查');button('撤销').click();await wait();
  button('提示').click();await wait();const highlighted=cells().findIndex(c=>c.classList.contains('hint'));const targetBlank=highlighted>=0&&cells()[highlighted].getAttribute('aria-label').includes('空')&&cells()[highlighted].classList.contains('hint-blue')&&!document.querySelector('.hint-unit')&&!document.querySelector('.candidate-highlight,.candidate-remove');
  button('提示').click();await wait();const technique=/Full House|Naked Single|Hidden Single/.test(document.querySelector('.message').textContent)&&!document.querySelector('.hint-unit');
  button('提示').click();await wait();const preview=cells()[highlighted].getAttribute('aria-label').includes('空')&&document.querySelectorAll('.hint-unit').length>0&&cells()[highlighted].querySelector('.candidate-highlight')?.textContent===answer[highlighted]&&cells()[highlighted].getAttribute('aria-label').includes('提示相关');
  button('提示').click();await wait();const applied=cells()[highlighted].textContent.trim()===answer[highlighted];button('撤销').click();await wait();const undo=cells()[highlighted].getAttribute('aria-label').includes('空');
  cells()[0].click();await wait();key(4).click();await wait();
  document.querySelector('.back-button').click();await wait();Math.random=()=>0.2;document.querySelector('.easy-course').click();await wait();const prompt=document.querySelector('#restart-title').textContent.includes('简单');
  document.querySelector('.restart-cancel').click();await wait();document.querySelector('.continue-row').click();await wait();const cancelPreserved=cells()[0].textContent.trim()==='4';
  document.querySelector('.back-button').click();await wait();document.querySelector('.easy-course').click();await wait();document.querySelector('.restart-confirm').click();await wait();
  const secondSaved=JSON.parse(localStorage.getItem('sudoku402.game.v1')).puzzle==='s401-10574';
  const secondBoard=cells()[0].textContent.trim()==='8'&&cells()[1].getAttribute('aria-label').includes('空');
  cells()[1].click();await wait();key(6).click();await wait();const solutionSwitched=cells()[1].textContent.trim()==='6'&&!cells()[1].classList.contains('conflict');
  return {entered,firstSaved,wrong,errorFirst,targetBlank,technique,preview,applied,undo,prompt,cancelPreserved,secondSaved,secondBoard,solutionSwitched};
 })()`});
 const checks=r.result?.result?.value;if(!checks)throw Error(JSON.stringify(r));await call('Page.reload');await new Promise(r=>setTimeout(r,2500));
 const reload=await call('Runtime.evaluate',{awaitPromise:true,returnByValue:true,expression:`(async()=>{document.querySelector('.continue-row').click();await new Promise(r=>setTimeout(r,100));return document.querySelector('.game-header h1').textContent==='简单'&&document.querySelectorAll('[role=gridcell]')[1].textContent.trim()==='6'})()`});checks.resumed=reload.result.result.value;
 console.log(JSON.stringify(checks));ws.close();if(Object.values(checks).some(v=>!v))process.exitCode=1;
}
main().catch(e=>{console.error(e);process.exitCode=1});
