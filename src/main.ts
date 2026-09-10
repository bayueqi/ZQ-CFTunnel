import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import { initSoundDelegation } from "./utils/sound";

// 初始化全局 Web Audio 音效事件代理
initSoundDelegation();

createApp(App).mount("#app");


