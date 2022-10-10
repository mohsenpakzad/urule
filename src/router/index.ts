import { route } from 'quasar/wrappers';
import { createRouter, createWebHistory } from 'vue-router';

import routes from './routes';
import { WebviewWindow } from '@tauri-apps/api/window';
import { tauri } from 'app/src-tauri/tauri.conf.json';

/*
 * If not building with SSR mode, you can
 * directly export the Router instantiation;
 *
 * The function below can be async too; either use
 * async/await or return a Promise which resolves
 * with the Router instance.
 */

export default route(function (/* { store, ssrContext } */) {
  const Router = createRouter({
    scrollBehavior: () => ({ left: 0, top: 0 }),
    routes,

    // Leave this as is and make changes in quasar.conf.js instead!
    // quasar.conf.js -> build -> vueRouterMode
    // quasar.conf.js -> build -> publicPath
    history: createWebHistory(),
  });

  const mainWindow = WebviewWindow.getByLabel('main');
  const originalWindowTitle = tauri.windows[0].title;
  Router.afterEach(async (to) => {
    if (to.name) {
      await mainWindow?.setTitle(
        `${originalWindowTitle} - ${to.name.toString()}`
      );
    } else {
      await mainWindow?.setTitle(originalWindowTitle);
    }
  });

  return Router;
});
