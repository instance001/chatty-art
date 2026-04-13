import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "./App.vue";
import router from "./router";

import 'element-plus/theme-chalk/dark/css-vars.css'
import "./assets/main.css";
import { useGeneratorStore } from "./stores/generator";

const app = createApp(App);

app.use(createPinia());
app.use(router);

app.mount("#app");
router.replace("/");

window.addEventListener("beforeunload", (event) => {
    let busy = useGeneratorStore().generating;
    if(busy)
    {
        event.preventDefault();
        event.returnValue = "";
    }
});