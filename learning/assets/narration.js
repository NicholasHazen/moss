(async function () {
  "use strict";
  const player = document.querySelector("[data-narration-source]");
  if (!player) return;
  const audio = player.querySelector("audio");
  const status = player.querySelector("[data-narration-status]");
  let active;
  let useAudioHighlight = true;
  try {
    const response = await fetch(player.dataset.narrationSource);
    if (!response.ok) throw new Error("missing cues");
    const metadata = await response.json();
    const toggle = document.createElement("button");
    toggle.type="button";toggle.textContent="Play audio";
    document.querySelector(".reading-toolbar").append(toggle);
    const followLabel=document.createElement("label");
    const follow=document.createElement("input");follow.type="checkbox";follow.checked=true;
    followLabel.append(follow," Follow the highlighted passage");player.append(followLabel);
    toggle.addEventListener("click",()=>{
      if (audio.paused) audio.play().catch(()=>{status.textContent="Playback did not start. Try the native audio controls.";});
      else audio.pause();
    });
    const clear = () => {active?.classList.remove("narration-active");active=null;};
    const sync = () => {
      if (!useAudioHighlight) {clear();return;}
      const cue = metadata.cues.find(c => c.start <= audio.currentTime && c.end > audio.currentTime);
      if (!cue) {clear();return;}
      const next = document.querySelector(`[data-narration="${cue.passage}"]`);
      if (next !== active) {
        clear(); active=next; active?.classList.add("narration-active");
        if (!audio.paused && follow.checked) active?.scrollIntoView({block:"center",behavior:"auto"});
      }
      const message=`Passage ${cue.passage+1} of ${metadata.cues.length}. ${audio.paused ? "Paused." : "Playing."}`;
      if (status.textContent !== message) status.textContent=message;
    };
    audio.addEventListener("timeupdate",sync);
    audio.addEventListener("seeked",sync);
    audio.addEventListener("play",()=>{useAudioHighlight=true;const stop=document.getElementById("stop-reading");if (!stop.hidden) stop.click();toggle.textContent="Pause audio";sync();});
    audio.addEventListener("pause",()=>{toggle.textContent="Resume audio";status.textContent="Narration paused. Resume or seek with the audio controls.";});
    audio.addEventListener("ended",()=>{clear();toggle.textContent="Play audio";status.textContent="Narration finished.";});
    audio.addEventListener("error",()=>{useAudioHighlight=false;clear();status.textContent="Audio could not be played. The complete lesson remains below.";});
    document.getElementById("listen").addEventListener("click",()=>{useAudioHighlight=false;audio.pause();clear();});
    status.textContent = `Ready. ${Math.ceil(metadata.duration/60)} minutes at the generated pace. Use the audio menu to change playback speed.`;
  } catch (_) {status.textContent="Passage cues unavailable; the audio controls and lesson remain available.";}
})();
