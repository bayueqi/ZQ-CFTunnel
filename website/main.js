// 导航吸顶状态
const nav = document.getElementById("nav");
const onScroll = () => nav.classList.toggle("is-scrolled", window.scrollY > 6);
onScroll();
window.addEventListener("scroll", onScroll, { passive: true });

// 滚动入场：元素进入视口即淡入，只处理尚未显示的那些
const pending = Array.from(document.querySelectorAll(".reveal"));
let ticking = false;

const revealVisible = () => {
  ticking = false;
  const limit = window.innerHeight - 40;
  for (let i = pending.length - 1; i >= 0; i--) {
    if (pending[i].getBoundingClientRect().top < limit) {
      pending[i].classList.add("is-in");
      pending.splice(i, 1);
    }
  }
};

revealVisible();
window.addEventListener(
  "scroll",
  () => {
    if (ticking || !pending.length) return;
    ticking = true;
    requestAnimationFrame(revealVisible);
  },
  { passive: true }
);
window.addEventListener("resize", revealVisible);
window.addEventListener("load", revealVisible);

// 锚点跳转、动画期间没有滚动事件时兜底
const watcher = setInterval(() => {
  revealVisible();
  if (!pending.length) clearInterval(watcher);
}, 600);
