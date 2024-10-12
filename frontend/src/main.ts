import { Manager } from "./Manager";

window.onload = () => {
  let ctx: AudioContext;
  const start = document.getElementById("start") as HTMLDivElement;
  const startscreen = document.getElementById("startscreen") as HTMLDivElement;

  const manager = new Manager();

  start.addEventListener("click", async () => {
    startscreen.style.display = "none";
    ctx = new AudioContext();
    manager.init(ctx);
  });

  start.addEventListener("touchstart", async () => {
    startscreen.style.display = "none";
    ctx = new AudioContext();
    manager.init(ctx);
  });

  start.addEventListener("touchend", () => {
    ctx.resume();
  });
};
