// Fresh Brave CDP :19222 and built app served on :18765.
const fixture=require('./locked-fixture.json');
async function main(){
 let pages;for(let i=0;i<80;i++){try{pages=await(await fetch('http://127.0.0.1:19222/json')).json();if(pages.length)break}catch{}await new Promise(r=>setTimeout(r,250))}
 if(!pages?.length)throw Error('No Brave target');const ws=new WebSocket(pages[0].webSocketDebuggerUrl);await new Promise((ok,fail)=>{ws.onopen=ok;ws.onerror=fail});
 let id=0;const pending=new Map();ws.onmessage=e=>{const m=JSON.parse(e.data);if(pending.has(m.id)){pending.get(m.id)(m);pending.delete(m.id)}};
 const call=(method,params={})=>new Promise(resolve=>{const n=++id;pending.set(n,resolve);ws.send(JSON.stringify({id:n,method,params}))});
 const evaluate=async expression=>{const r=await call('Runtime.evaluate',{awaitPromise:true,returnByValue:true,expression});if(!r.result?.result?.value)throw Error(JSON.stringify(r));return r.result.result.value};
 await call('Page.enable');await call('Runtime.enable');await call('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:3,mobile:true});
 await call('Page.navigate',{url:'http://127.0.0.1:18765/'});await new Promise(r=>setTimeout(r,3500));
 async function load(change,data){
  const save=data?structuredClone(data):{version:1,puzzle:'s401-'+fixture.id,values:[...fixture.values],candidates:[...fixture.candidates],auto_candidates:true};change?.(save);
  await call('Runtime.evaluate',{expression:`localStorage.setItem('sudoku402.game.v1',${JSON.stringify(JSON.stringify(save))})`});await call('Page.reload');await new Promise(r=>setTimeout(r,2500));
  await call('Runtime.evaluate',{expression:`document.querySelector('.continue-row').click()`});await new Promise(r=>setTimeout(r,100));
 }
 const checks={};await load(undefined,require('./medium-fixtures.json').Locked_Candidates);
 Object.assign(checks,await evaluate(`(async()=>{const w=()=>new Promise(r=>setTimeout(r,100));const c=()=>[...document.querySelectorAll('[role=gridcell]')];const hint=()=>[...document.querySelectorAll('.controls button')].find(x=>x.textContent==='提示');const before=localStorage.getItem('sudoku402.game.v1');
 hint().click();await w();const first=c().some(x=>x.classList.contains('hint-blue'))&&!c().some(x=>x.classList.contains('hint-unit'))&&!document.querySelector('.candidate-highlight,.candidate-remove');
 hint().click();await w();const second=document.querySelector('.message').textContent.includes('Locked Candidates')&&!c().some(x=>x.classList.contains('hint-unit'))&&!document.querySelector('.candidate-highlight,.candidate-remove');
 hint().click();await w();const blues=[...document.querySelectorAll('.candidate-highlight')], reds=[...document.querySelectorAll('.candidate-remove')];const third=c().filter(x=>x.classList.contains('hint-unit')).length===15&&blues.length>0&&reds.length>0&&blues.every(x=>x.closest('.cell').classList.contains('hint-green-fill'))&&reds.every(x=>x.closest('.cell').classList.contains('hint-orange'));const accessible=blues.every(x=>x.closest('.cell').getAttribute('aria-label').includes('提示相关'))&&reds.every(x=>x.closest('.cell').getAttribute('aria-label').includes('待移除'));const unchanged=before===localStorage.getItem('sudoku402.game.v1');
 hint().click();await w();const after=localStorage.getItem('sudoku402.game.v1');const applied=before!==after;[...document.querySelectorAll('.controls button')].find(x=>x.textContent==='撤销').click();await w();const undo=before===localStorage.getItem('sudoku402.game.v1');return {first,second,third,accessible,unchanged,applied,undo}})()`));
 const i=fixture.hits[0],digit=fixture.digit;
 await load(s=>{s.candidates[i]&=~(1<<digit)});
 Object.assign(checks,await evaluate(`(async()=>{const w=()=>new Promise(r=>setTimeout(r,100));const c=()=>document.querySelectorAll('[role=gridcell]');const h=()=>[...document.querySelectorAll('.controls button')].find(x=>x.textContent==='提示');const before=localStorage.getItem('sudoku402.game.v1');h().click();await w();const missingEarly=c()[${i}].classList.contains('hint-blue');h().click();await w();const missingTitle=document.querySelector('.message').textContent.includes('Incorrect Pencilmark');h().click();await w();const missingPreview=c()[${i}].classList.contains('hint-green-outline')&&!c()[${i}].querySelector('.candidate-highlight')&&document.querySelector('.message').textContent.includes('缺少')&&before===localStorage.getItem('sudoku402.game.v1');h().click();await w();const missingApply=!!(JSON.parse(localStorage.getItem('sudoku402.game.v1')).candidates[${i}]&(1<<${digit}));[...document.querySelectorAll('.controls button')].find(x=>x.textContent==='撤销').click();await w();return {missingEarly,missingTitle,missingPreview,missingApply,missingUndo:before===localStorage.getItem('sudoku402.game.v1')}})()`));
 await load(s=>{s.candidates[i]|=1<<5});
 Object.assign(checks,await evaluate(`(async()=>{const w=()=>new Promise(r=>setTimeout(r,100));const h=()=>[...document.querySelectorAll('.controls button')].find(x=>x.textContent==='提示');for(let j=0;j<3;j++){h().click();await w()}const cell=document.querySelectorAll('[role=gridcell]')[${i}];const wrongNote=cell.classList.contains('hint-orange')&&[...cell.querySelectorAll('.candidate-remove')].some(x=>x.textContent==='5')&&cell.getAttribute('aria-label').includes('待移除');return {wrongNote}})()`));
 console.log(JSON.stringify(checks));ws.close();if(Object.values(checks).some(v=>!v))process.exitCode=1;
}main().catch(e=>{console.error(e);process.exitCode=1});
