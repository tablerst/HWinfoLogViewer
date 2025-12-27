<template>
  <n-layout has-sider class="main-layout">
    <n-layout-sider
        bordered
        collapse-mode="width"
        :collapsed-width="64"
        :width="240"
        :collapsed="collapsed"
        show-trigger
        @collapse="collapsed = true"
        @expand="collapsed = false"
    >
      <SideBar/>
    </n-layout-sider>
    <n-layout-content class="content-layout">
      <router-view/>
    </n-layout-content>
  </n-layout>
</template>

<script setup lang="ts">
import {ref} from "vue";
import SideBar from "../components/SideBar.vue";

const collapsed = ref(false)
</script>

<style scoped>
.main-layout {
  height: 100vh;
}

.content-layout {
  padding: 24px;
  background-color: #f5f7f9;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

@media (prefers-color-scheme: dark) {
  .content-layout {
    background-color: #101014;
  }
}
</style>
