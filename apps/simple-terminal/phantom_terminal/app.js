const log = document.querySelector('#log');
const status = document.querySelector('#status');
const meter = document.querySelector('#meterFill');
const listen = document.querySelector('#listen');
const risk = document.querySelector('#risk');
const reset = document.querySelector('#reset');
const jumpscare = document.querySelector('#jumpscare');
const closeScare = document.querySelector('#closeScare');

const messages = [
  ['> Porteuse détectée.', 'normal'],
  ['> Analyse des impulsions…', 'normal'],
  ['> Ce motif ne ressemble à aucun signal connu.', 'normal'],
  ['> ...', 'muted'],
  ['> TU ES ENCORE LÀ ?', 'warning'],
  ['> Réponse reçue : 1 utilisateur.', 'warning'],
  ['> Attends. Il y a un deuxième écho.', 'warning'],
  ['> FIN DE TRANSMISSION.', 'warning']
];
let running = false;
let timers = [];

function addLine(text, cls='') {
  const p = document.createElement('p');
  p.textContent = text;
  if (cls) p.className = cls;
  log.appendChild(p);
}
function clearTimers(){ timers.forEach(clearTimeout); timers=[]; }

function start(){
  if(running) return;
  running = true;
  document.body.classList.add('active');
  listen.disabled = true;
  listen.textContent = 'SIGNAL EN COURS…';
  status.textContent = 'RÉCEPTION';
  log.replaceChildren();
  meter.style.width = '8%';
  messages.forEach(([text,cls],i)=>{
    timers.push(setTimeout(()=>{
      addLine(text,cls);
      meter.style.width = `${Math.min(100, 12 + (i+1)*12)}%`;
      if(i===4) status.textContent = 'ANOMALIE';
      if(i===messages.length-1){
        status.textContent = 'SILENCE';
        listen.disabled = false;
        listen.textContent = 'RÉÉCOUTER';
        running = false;
        document.body.classList.remove('active');
      }
    }, 850*(i+1)));
  });
}

function trySignal(){
  // Une chance sur dix : les nombres 0 à 9 sont équiprobables.
  if (Math.floor(Math.random() * 10) === 0) {
    jumpscare.classList.add('show');
    jumpscare.setAttribute('aria-hidden', 'false');
    status.textContent = 'SIGNAL INATTENDU';
  } else {
    status.textContent = 'AUCUNE RÉPONSE';
    addLine('> Rien… pour cette fois.', 'muted');
  }
}
function hideScare(){
  jumpscare.classList.remove('show');
  jumpscare.setAttribute('aria-hidden', 'true');
  status.textContent = 'EN ATTENTE';
}
function restart(){
  clearTimers();
  running=false;
  hideScare();
  document.body.classList.remove('active');
  listen.disabled=false;
  listen.textContent='ÉCOUTER LE SIGNAL';
  status.textContent='EN ATTENTE';
  meter.style.width='4%';
  log.replaceChildren();
  addLine('> Initialisation du récepteur...');
  addLine('> Bruit détecté sur la fréquence 03.17.');
  addLine('> Appuyez sur « ÉCOUTER » si vous osez.','muted');
}
listen.addEventListener('click',start);
risk.addEventListener('click',trySignal);
reset.addEventListener('click',restart);
closeScare.addEventListener('click',hideScare);
jumpscare.addEventListener('click',(event)=>{ if(event.target===jumpscare) hideScare(); });
document.addEventListener('keydown',(event)=>{ if(event.key==='Escape') hideScare(); });
