<template>
  <div :class="['fluent-window', isDarkMode ? 'dark-theme' : 'light-theme', { 'is-maximized': isMaximized }]">
    <!-- Win11 顶部无边框亚克力自定义标题栏 (最高层级 z-index: 5000, 支持原生拖拽 & 统一尺寸按钮) -->
    <header class="fluent-header" data-tauri-drag-region>
      <div class="header-left">
        <!-- Cloudflared Logo (带悬浮放大与点击抖动随机播放酒狐彩蛋音效) -->
        <div
          class="app-logo-wrapper"
          :class="{ shake: isLogoShaking }"
          @click.stop="onLogoClick"
          :title="t.chrome.logo_title"
        >
          <img src="/cloudflared.ico" alt="Cloudflared Logo" class="app-logo" />
        </div>
        <div class="app-title-group" data-tauri-drag-region>
          <h1 class="app-title" data-tauri-drag-region>{{ t.header }}</h1>
        </div>
      </div>

      <div class="header-center" data-tauri-drag-region></div>

      <div class="header-right">
        <!-- 1. 音效开关按钮 -->
        <button
          class="fluent-icon-btn sound-btn"
          @click="toggleSound"
          :title="isSoundEnabled ? t.chrome.sound_mute_title : t.chrome.sound_unmute_title"
        >
          <span :class="['sound-icon', { spin: isSoundSpinning }]">
            {{ isSoundEnabled ? '🔊' : '🔇' }}
          </span>
        </button>

        <!-- 2. GitHub 仓库链接按钮 (指向 bayueqi/ZQ-CFTunnel) -->
        <button
          class="fluent-icon-btn github-btn"
          @click="openUrl('https://github.com/bayueqi/ZQ-CFTunnel')"
          :title="t.chrome.github_title"
        >
          <svg class="github-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path fill="currentColor" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z"/>
          </svg>
        </button>

        <!-- 3. 主题切换按钮 -->
        <button
          class="fluent-icon-btn theme-btn"
          @click="toggleTheme"
          :title="isDarkMode ? t.chrome.theme_to_light : t.chrome.theme_to_dark"
        >
          <span :class="['theme-icon', { spin: isThemeSpinning }]">
            {{ isDarkMode ? '☀' : '🌙' }}
          </span>
        </button>

        <!-- 4. 最小化按钮 (统一 28x28 尺寸，通过后端原生命令可靠最小化) -->
        <button
          class="fluent-icon-btn win-ctrl-btn minimize-btn"
          @click.stop="handleMinimize"
          :title="t.chrome.btn_minimize"
        >
          <svg class="win-ctrl-icon" width="10" height="1" viewBox="0 0 10 1">
            <rect width="10" height="1" fill="currentColor" />
          </svg>
        </button>

        <!-- 5. 最大化 / 还原按钮 (统一 28x28 尺寸，通过后端原生命令可靠缩放) -->
        <button
          class="fluent-icon-btn win-ctrl-btn maximize-btn"
          @click.stop="handleToggleMaximize"
          :title="isMaximized ? t.chrome.btn_restore : t.chrome.btn_maximize"
        >
          <svg v-if="!isMaximized" class="win-ctrl-icon" width="10" height="10" viewBox="0 0 10 10">
            <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1" />
          </svg>
          <svg v-else class="win-ctrl-icon" width="10" height="10" viewBox="0 0 10 10">
            <path d="M2.5 0.5H9.5V7.5M0.5 2.5H7.5V9.5H0.5Z" fill="none" stroke="currentColor" stroke-width="1" />
          </svg>
        </button>

        <!-- 6. 关闭按钮 (统一 28x28 尺寸，通过后端原生命令优雅隐藏到系统托盘) -->
        <button
          class="fluent-icon-btn win-ctrl-btn close-btn"
          @click.stop="handleCloseWindow"
          :title="t.chrome.btn_close"
        >
          <svg class="win-ctrl-icon" width="10" height="10" viewBox="0 0 10 10">
            <line x1="0.5" y1="0.5" x2="9.5" y2="9.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            <line x1="9.5" y1="0.5" x2="0.5" y2="9.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
        </button>
      </div>
    </header>

    <!-- 双栏布局：左侧可折叠侧边栏 + 主体内容区 -->
    <div class="app-layout">
      <!-- ============ 左侧侧边栏 ============ -->
      <aside :class="['fluent-sidebar', { collapsed: sidebarCollapsed }]">
        <!-- 侧边栏整体展开/折叠按钮 -->
        <button
          class="sidebar-toggle"
          @click="sidebarCollapsed = !sidebarCollapsed"
          :title="sidebarCollapsed ? t.chrome.sidebar_expand : t.chrome.sidebar_collapse"
        >
          <span class="toggle-icon">{{ sidebarCollapsed ? '☰' : '⟨⟨' }}</span>
          <span v-if="!sidebarCollapsed" class="toggle-text">{{ t.chrome.sidebar_collapse_text }}</span>
        </button>

        <!-- 1. 服务端（含子级：临时链接 / 固定域名 / 云端托管） -->
        <div class="sidebar-group">
          <button
            :class="['sidebar-item', { active: currentTab === 'server', expanded: sidebarOpen.server }]"
            @click="handleSidebarClick('server')"
            :title="t.tabs.server"
          >
            <span class="sidebar-icon">🖥️</span>
            <span class="sidebar-text">{{ t.tabs.server }}</span>
            <span class="sidebar-arrow">▸</span>
          </button>

          <!-- 服务端下边栏（默认折叠） -->
          <transition name="sub-list">
            <div v-show="sidebarOpen.server && !sidebarCollapsed" class="sidebar-sub-list">
              <button
                :class="['sidebar-sub-item', { active: currentTab === 'server' && isServerView('quick') }]"
                @click="switchServerView('quick')"
                :title="t.server_tab.nav_quick"
              >
                <span class="mode-dot quick"></span>
                <span class="sidebar-text">{{ t.server_tab.nav_quick }}</span>
              </button>

              <button
                :class="['sidebar-sub-item', { active: currentTab === 'server' && isServerView('named') }]"
                @click="switchServerView('named')"
                :title="t.server_tab.nav_named"
              >
                <span class="mode-dot local"></span>
                <span class="sidebar-text">{{ t.server_tab.nav_named }}</span>
              </button>
            </div>
          </transition>
        </div>

        <!-- 2. 客户端 -->
        <div class="sidebar-group">
          <button
            :class="['sidebar-item', { active: currentTab === 'client' }]"
            @click="handleSidebarClick('client')"
            :title="t.tabs.client"
          >
            <span class="sidebar-icon">💻</span>
            <span class="sidebar-text">{{ t.tabs.client }}</span>
          </button>
        </div>

        <!-- 3. 配置 -->
        <div class="sidebar-group">
          <button
            :class="['sidebar-item', { active: currentTab === 'misc' }]"
            @click="handleSidebarClick('misc')"
            :title="t.tabs.misc"
          >
            <span class="sidebar-icon">⚙️</span>
            <span class="sidebar-text">{{ t.tabs.misc }}</span>
          </button>
        </div>
      </aside>

      <!-- 主体内容卡片区 (平滑过渡动效) -->
      <main class="fluent-body">
      <!-- 1. 服务端 Tab -->
      <!-- 视图容器用 v-if 而不是 v-show：v-show 会让三个 Tab、五个子视图的 DOM 全部长期常驻，
           隐藏页面的节点既占内存，也参与每个响应式更新周期的 diff。这里各视图天然互斥，
           改用 v-if 后只挂载当前可见的那一份，其余整棵卸载。表单值都走 ref / localStorage，切换不丢。 -->
      <section v-if="currentTab === 'server'" class="tab-view server-view animated-view">
        <!-- 服务端只有两个子视图：临时隧道 / 固定隧道。
             原「云端托管」视图已并入固定隧道 —— 隧道在云端还是本地托管，是它的属性，
             不该是另一个列表（同一个隧道会因为它而出现在两处）。 -->
        <div class="server-sub-view">
          <!-- ============ 临时隧道 ============ -->
          <div v-if="localSubMode === 'quick'" class="server-sub-view">
          <!-- 运行中的临时链接列表卡片：创建入口收进右上角，不再放常驻表单 -->
          <div class="fluent-card table-card">
            <div class="client-title-row">
              <h3 class="card-title client-title">{{ t.server_tab.quick_list_title }}</h3>
              <div class="client-title-actions">
                <button class="fluent-btn small primary" @click="openQuickCreateModal">
                  <span class="btn-icon">＋</span>
                  {{ t.server_tab.btn_create }}
                </button>
                <button class="fluent-btn small" @click="refreshQuickTunnels" :disabled="refreshingTunnels.quick">
                  <span class="btn-icon" :class="{ spinning: refreshingTunnels.quick }">🔄</span>
                  {{ refreshingTunnels.quick ? t.server_tab.btn_refreshing : t.server_tab.btn_refresh }}
                </button>
                <div class="status-pill" :class="quickRunning ? 'online' : 'offline'">
                  {{ quickRunning
                    ? `${t.server_tab.status_running} (${quickTunnels.length})`
                    : t.server_tab.status_stopped }}
                </div>
              </div>
            </div>

            <div class="quick-list">
              <div v-if="quickTunnels.length === 0" class="quick-list-empty">
                {{ t.server_tab.quick_list_empty }}
              </div>
              <div v-for="qt in quickTunnels" :key="qt.key" class="quick-item">
                <div class="quick-item-head">
                  <span class="quick-item-target mono">{{ quickTargetLabel(qt) }}</span>
                  <span class="type-badge" :class="qt.status === 'running' ? 'type-local' : 'type-remote'">
                    {{ qt.status === 'running' ? t.server_tab.quick_status_online : t.server_tab.quick_status_starting }}
                  </span>
                </div>
                <div class="quick-item-url">
                  <span
                    v-if="qt.url"
                    class="quick-url-value mono"
                    :title="t.server_tab.click_to_copy"
                    @click="copyQuickUrl(qt.url)"
                  >{{ qt.url }}</span>
                  <span v-else class="quick-url-empty">{{ t.server_tab.quick_url_empty }}</span>
                </div>
                <div class="quick-item-actions">
                  <button v-if="qt.url" class="fluent-btn small primary" @click="openUrl(qt.url)">🌐 {{ t.server_tab.btn_open }}</button>
                  <button class="fluent-btn small danger" @click="promptStopQuick(qt.key)">⏹ {{ t.server_tab.quick_stop }}</button>
                </div>
              </div>
            </div>
          </div>
          </div>

          <!-- ============ 固定隧道 ============ -->
          <div v-if="localSubMode === 'named'" class="server-sub-view">
          <!-- 固定隧道列表卡片：创建 / 刷新 / 运行中(N) 收进右上角，
               每行的启停、修改、删除都做进行内操作 -->
          <div class="fluent-card table-card">
            <div class="client-title-row">
              <h3 class="card-title client-title">{{ t.server_tab.local_list_title }}</h3>
              <div class="client-title-actions">
                <button class="fluent-btn small primary" @click="openTunnelCreateModal">
                  <span class="btn-icon">＋</span>
                  {{ t.server_tab.btn_create }}
                </button>
                <button
                  class="fluent-btn small"
                  :disabled="refreshingTunnels.local"
                  @click="handleRefreshTunnels()"
                >
                  <span class="btn-icon" :class="{ spinning: refreshingTunnels.local }">🔄</span>
                  {{ refreshingTunnels.local ? t.server_tab.btn_refreshing : t.server_tab.btn_refresh }}
                </button>
                <div class="status-pill" :class="localRunningCount > 0 ? 'online' : 'offline'">
                  {{ localRunningCount > 0
                    ? `${t.server_tab.status_running} (${localRunningCount})`
                    : t.server_tab.status_stopped }}
                </div>
              </div>
            </div>

            <div class="fluent-table-wrapper">
              <table class="fluent-table">
                <thead>
                  <tr>
                    <th class="col-id">{{ t.server_tab.headers.id }}</th>
                    <th class="col-name">{{ t.server_tab.headers.name }}</th>
                    <th class="col-created">{{ t.server_tab.headers.created }}</th>
                    <th class="col-hostname">{{ t.server_tab.headers.hostname }}</th>
                    <th class="col-connections">{{ t.server_tab.headers.connections }}</th>
                    <th class="col-status">{{ t.server_tab.headers.status }}</th>
                    <th class="col-actions">{{ t.server_tab.headers.actions }}</th>
                  </tr>
                </thead>
                <tbody>
                  <!-- 这一份是账号下的全部隧道：不再按「本机有没有凭据文件」拆成两个列表 ——
                       那个判定跟「是不是云端托管」是两回事，会让同一条隧道出现在两处、
                       或明明云端托管却标成「本地」。 -->
                  <tr
                    v-for="tunnel in serverTunnelList"
                    :key="tunnel.id"
                    :class="{ selected: selectedTunnel?.id === tunnel.id }"
                    @click="selectTunnel(tunnel)"
                  >
                    <td class="col-id mono" :title="tunnel.id">{{ tunnel.id }}</td>
                    <td class="col-name font-bold">{{ tunnel.name }}</td>
                    <td class="col-created mono">{{ tunnel.created }}</td>
                    <td class="col-hostname">
                      <template v-if="tunnel.hostnames && tunnel.hostnames.length">
                        <span
                          class="hostname-tag"
                          v-for="h in tunnel.hostnames"
                          :key="h.id"
                          :title="t.server_tab.click_to_copy"
                          @click.stop="copyHostname(h.name)"
                        >{{ h.name }}</span>
                      </template>
                      <span v-else class="hostname-empty">{{ t.server_tab.hostname_unbound }}</span>
                    </td>
                    <td class="col-connections">{{ tunnel.connections || '-' }}</td>
                    <td class="col-status">
                      <span class="tunnel-status" :class="{ online: isTunnelRunning(tunnel.name) }">
                        <span class="status-dot" :class="isTunnelRunning(tunnel.name) ? 'green' : 'gray'"></span>
                        {{ isTunnelRunning(tunnel.name) ? t.server_tab.status_running : t.server_tab.status_not_running }}
                      </span>
                    </td>
                    <td class="col-actions">
                      <button
                        class="row-action-btn"
                        :class="isTunnelRunning(tunnel.name) ? 'danger' : 'primary'"
                        :title="isTunnelRunning(tunnel.name) ? t.server_tab.btn_stop : t.server_tab.btn_start"
                        @click.stop="isTunnelRunning(tunnel.name) ? handleStopServer(tunnel.name) : handleRowStart(tunnel)"
                      >
                        <span v-if="isTunnelRunning(tunnel.name)" class="icon-square"></span>
                        <span v-else class="icon-triangle"></span>
                      </button>
                      <button
                        class="row-action-btn"
                        :title="t.server_tab.named_edit_title"
                        @click.stop="openTunnelEditModal(tunnel)"
                      >✎</button>
                      <button
                        class="row-action-btn danger"
                        :title="t.server_tab.btn_delete"
                        @click.stop="promptDeleteTunnel(tunnel)"
                      >🗑</button>
                    </td>
                  </tr>
                  <tr v-if="serverTunnelList.length === 0">
                    <td colspan="7" class="empty-table">
                      {{ refreshingTunnels.local ? t.server_tab.table_refreshing : t.server_tab.table_empty }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- DNS 路由绑定卡片：标题行 + 添加按钮 + 按隧道分组的域名列表。
               分组层级：隧道（组容器）> 域名（组内区块）> 该域名的设置（区块内的行）。
               每个域名是一个自洽区块：上排是域名本身与域名级操作（改名 / 解绑），
               下排是这个域名的「访问密码锁」—— 锁是域名的属性，跟它放在一起看才不会错位。
               原「隧道列 / 域名列 / 密码列」三列表格已废弃：client id 32 位 + secret 40 位
               这类超长凭据塞进任何固定列宽都会折行，且与所属域名分处两列，看不出对应关系。 -->
          <div class="fluent-card form-card dns-route-card">
            <div class="dns-route-title-row">
              <h3 class="card-title dns-route-title">{{ t.server_tab.dns_section }}</h3>
              <button class="fluent-btn small primary" @click="openDnsAddModal">
                <span class="btn-icon">＋</span>
                {{ t.server_tab.dns_add_btn }}
              </button>
            </div>

            <!-- 已绑定域名管理：只列出固定域名的绑定记录，按隧道分组 -->
            <div class="dns-group-list">
              <div v-for="group in dnsBoundGroups" :key="group.tunnelId" class="dns-group">
                <div class="dns-group-head">
                  <span class="dns-group-name" :title="group.tunnelName">{{ group.tunnelName }}</span>
                  <span class="dns-group-count">
                    {{ t.server_tab.dns_group_count.replace('{count}', String(group.records.length)) }}
                  </span>
                </div>

                <div
                  v-for="rec in group.records"
                  :key="rec.recordId"
                  class="dns-domain"
                  :class="{ locked: !!lockOf(rec.hostname) }"
                >
                  <div class="dns-domain-row">
                    <span
                      class="dns-domain-host"
                      :title="rec.hostname + ' · ' + t.server_tab.click_to_copy"
                      @click="copyHostname(rec.hostname)"
                    >{{ rec.hostname }}</span>
                    <span class="dns-row-actions">
                      <button
                        class="dns-mini-btn"
                        :title="t.server_tab.dns_edit_title"
                        @click.stop="promptEditDnsRoute(rec)"
                      >{{ t.server_tab.dns_btn_rename }}</button>
                      <button
                        class="dns-mini-btn danger"
                        :title="t.server_tab.btn_unbind"
                        @click.stop="promptUnbindDnsRoute(rec)"
                      >{{ t.server_tab.dns_btn_unbind }}</button>
                    </span>
                  </div>

                  <!-- 该域名的访问密码锁设置：状态徽标 + 掩码凭据 + 锁操作。
                       展开「显示凭据」时凭据不留在这一行 —— 32 位账号 + 40 位密码会把这一行
                       折成三四段、按钮被挤到最底下。展开态改由下面独立的整行块承载。 -->
                  <div class="dns-domain-row dns-lock-row">
                    <span class="dns-lock-label">{{ t.server_tab.headers.lock }}</span>

                    <template v-if="lockOf(rec.hostname)">
                      <span class="dns-lock-state on">{{ t.server_tab.lock_on }}</span>
                      <template v-if="!isLockCredExpanded(rec.hostname)">
                        <span
                          class="dns-cred"
                          :title="t.server_tab.lock_account_label + ' · ' + t.server_tab.click_to_copy"
                          @click="copyText(lockOf(rec.hostname)?.clientId || '')"
                        >{{ credDisplay(lockOf(rec.hostname)?.clientId || '', rec.hostname) }}</span>
                        <span
                          class="dns-cred"
                          :title="t.server_tab.lock_secret_label + ' · ' + t.server_tab.click_to_copy"
                          @click="copyText(lockOf(rec.hostname)?.clientSecret || '')"
                        >{{ credDisplay(lockOf(rec.hostname)?.clientSecret || '', rec.hostname) }}</span>
                      </template>
                      <span class="dns-row-actions">
                        <button
                          class="dns-mini-btn"
                          @click.stop="toggleLockCred(rec.hostname)"
                        >{{ isLockCredExpanded(rec.hostname) ? t.server_tab.lock_cred_hide : t.server_tab.lock_cred_show }}</button>
                        <button
                          class="dns-mini-btn"
                          :title="t.server_tab.btn_rotate_password"
                          :disabled="isLockMutating"
                          @click.stop="promptRotatePassword(rec.hostname)"
                        >{{ t.server_tab.btn_rotate_password }}</button>
                        <button
                          class="dns-mini-btn danger"
                          :title="t.server_tab.btn_unlock"
                          :disabled="isLockMutating"
                          @click.stop="promptUnlockHostname(rec.hostname)"
                        >{{ t.server_tab.btn_unlock }}</button>
                      </span>
                    </template>

                    <template v-else>
                      <span class="dns-lock-state off">{{ t.server_tab.lock_off }}</span>
                      <span class="dns-row-actions">
                        <button
                          class="dns-mini-btn primary"
                          :title="t.server_tab.btn_lock"
                          :disabled="isLockMutating"
                          @click.stop="promptLockHostname(rec.hostname)"
                        >{{ t.server_tab.btn_lock }}</button>
                      </span>
                    </template>
                  </div>

                  <!-- 展开的凭据：整行、账号与密码各占一行（标签 + 值，点击即复制）。
                       放在锁行下面而不是行内，按钮才不会被超长凭据挤到第三、四行 -->
                  <div
                    v-if="lockOf(rec.hostname) && isLockCredExpanded(rec.hostname)"
                    class="dns-cred-block"
                  >
                    <div class="dns-cred-item">
                      <span class="dns-cred-key">{{ t.server_tab.lock_account_label }}</span>
                      <span
                        class="dns-cred-val mono"
                        :title="t.server_tab.click_to_copy"
                        @click="copyText(lockOf(rec.hostname)?.clientId || '')"
                      >{{ lockOf(rec.hostname)?.clientId || '' }}</span>
                    </div>
                    <div class="dns-cred-item">
                      <span class="dns-cred-key">{{ t.server_tab.lock_secret_label }}</span>
                      <span
                        class="dns-cred-val mono"
                        :title="t.server_tab.click_to_copy"
                        @click="copyText(lockOf(rec.hostname)?.clientSecret || '')"
                      >{{ lockOf(rec.hostname)?.clientSecret || '' }}</span>
                    </div>
                  </div>
                </div>
              </div>

              <div v-if="dnsBoundGroups.length === 0" class="dns-empty">
                {{ t.server_tab.dns_bound_empty }}
              </div>
            </div>
          </div>
          </div>
        </div>

      </section>

      <!-- 2. 客户端 Tab：支持多开，一条隧道一行，可同时桥接多条 -->
      <section v-if="currentTab === 'client'" class="tab-view client-view animated-view">
        <div class="fluent-card form-card client-card">
          <div class="client-title-row">
            <h3 class="card-title client-title">{{ t.client_tab.title }}</h3>
            <div class="client-title-actions">
              <button class="fluent-btn small primary" @click="openClientAddModal">
                <span class="btn-icon">＋</span>
                {{ t.client_tab.add_btn }}
              </button>
              <button class="fluent-btn small" @click="refreshClientTunnels(true)">
                <span class="btn-icon">🔄</span>
                {{ t.client_tab.refresh_btn }}
              </button>
              <div class="status-pill" :class="clientRunningCount > 0 ? 'online' : 'offline'">
                {{ clientRunningCount > 0
                  ? `${t.client_tab.status_connected} ${clientRunningCount}`
                  : t.client_tab.status_disconnected }}
              </div>
            </div>
          </div>

          <!-- 客户端隧道列表：配置持久保存，断开后条目保留，可随时再启动 -->
          <div class="fluent-table-wrapper client-table-wrapper">
            <table class="fluent-table">
              <thead>
                <tr>
                  <th class="col-hostname">{{ t.client_tab.col_domain }}</th>
                  <th class="col-port">{{ t.client_tab.col_port }}</th>
                  <th class="col-password">{{ t.client_tab.col_password }}</th>
                  <th class="col-status">{{ t.client_tab.col_status }}</th>
                  <th class="col-actions">{{ t.client_tab.col_action }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="c in clientRows" :key="c.key">
                  <td class="col-hostname font-bold" :title="c.domain">{{ c.domain }}</td>
                  <td class="col-port mono">{{ c.port }}</td>
                  <!-- 已保存的访问凭据：明文展示，点击复制；未配置显示占位符 -->
                  <td class="col-password">
                    <div v-if="c.tokenId || c.tokenSecret" class="lock-cred">
                      <span
                        class="lock-cred-line mono"
                        :title="t.server_tab.lock_account_label + ' · ' + t.server_tab.click_to_copy"
                        @click="copyText(c.tokenId || '')"
                      >{{ c.tokenId || '—' }}</span>
                      <span
                        class="lock-cred-line mono"
                        :title="t.server_tab.lock_secret_label + ' · ' + t.server_tab.click_to_copy"
                        @click="copyText(c.tokenSecret || '')"
                      >{{ c.tokenSecret || '—' }}</span>
                    </div>
                    <span v-else class="lock-cred-empty">—</span>
                  </td>
                  <td class="col-status">
                    <span class="tunnel-status" :class="{ online: c.running }">
                      {{ c.running ? t.client_tab.status_connected : t.client_tab.status_disconnected }}
                    </span>
                  </td>
                  <td class="col-actions">
                    <button
                      class="row-action-btn"
                      :class="c.running ? 'danger' : 'primary'"
                      :title="c.running ? t.client_tab.btn_stop : t.client_tab.btn_start"
                      @click.stop="c.running ? handleStopClient(c) : handleStartClient(c)"
                    >
                      {{ c.running ? '⏹' : '▶' }}
                    </button>
                    <button
                      class="row-action-btn"
                      :title="t.client_tab.btn_edit"
                      :disabled="c.running"
                      @click.stop="openClientEditModal(c)"
                    >
                      ✎
                    </button>
                    <button
                      class="row-action-btn danger"
                      :title="t.client_tab.btn_delete"
                      @click.stop="handleDeleteClient(c)"
                    >
                      🗑
                    </button>
                  </td>
                </tr>
                <tr v-if="clientRows.length === 0">
                  <td colspan="5" class="empty-table">{{ t.client_tab.empty }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <!-- 3. 杂项 Tab (保留上下滑动 Slider) -->
      <section v-if="currentTab === 'misc'" class="tab-view misc-view animated-view">
        <!-- 访问密码锁凭证：上锁 / 解锁 / 换密码走 Cloudflare Access API 时使用。
             留空 = 使用「授权登录」的凭证；获取方式见项目 README。 -->
        <div class="fluent-card form-card access-token-card">
          <div class="access-token-row">
            <div class="fluent-form-group access-token-field">
              <label class="form-label">{{ t.misc_tab.access_token_label }}</label>
              <div class="input-container">
                <input
                  type="text"
                  v-model="accessTokenInput"
                  :placeholder="t.misc_tab.access_token_placeholder"
                  class="fluent-input mono"
                />
              </div>
            </div>
            <button class="fluent-btn primary access-token-save" @click="saveAccessToken">
              {{ t.server_tab.btn_save }}
            </button>
          </div>
          <div class="modal-hint">{{ t.misc_tab.access_token_hint }}</div>
        </div>

        <!-- 快捷操作区 -->
        <div class="fluent-card action-tiles-card">
          <div class="tile-grid">
            <!-- 1. 安装 cloudflared 按钮 -->
            <button class="tile-btn" @click="handleInstallCloudflared" :disabled="isDownloadingCloudflared">
              <span class="tile-icon">📦</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_install }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_install_desc }}</span>
              </div>
            </button>

            <!-- 2. 打开本地配置文件目录 -->
            <button class="tile-btn" @click="handleOpenConfigDir">
              <span class="tile-icon">📂</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_open_config_dir }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_open_config_dir_desc }}</span>
              </div>
            </button>

            <!-- 3. 授权登录按钮 -->
            <button class="tile-btn" @click="handleCloudflaredLogin">
              <span class="tile-icon">🔑</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_login }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_login_desc }}</span>
              </div>
            </button>

            <!-- 4. 检查版本按钮 -->
            <button class="tile-btn" @click="handleCheckVersion">
              <span class="tile-icon">ℹ️</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_check_version }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_check_version_desc }}</span>
              </div>
            </button>

            <!-- 5. 在线更新按钮 -->
            <button class="tile-btn" @click="handleUpdateCloudflared">
              <span class="tile-icon">⚡</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_update }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_update_desc }}</span>
              </div>
            </button>

            <!-- 6. 前往官网下载 -->
            <button class="tile-btn" @click="handleDownloadCloudflared">
              <span class="tile-icon">⬇️</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_download }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_download_desc }}</span>
              </div>
            </button>
          </div>

          <!-- 醒目的敏感凭证安全防泄露警告提示 (正下方红色醒目提示) -->
          <div class="config-warning-banner">
            <span class="warning-icon">⚠️</span>
            <span class="warning-text">{{ t.misc_tab.config_dir_warning }}</span>
          </div>
        </div>

      </section>
      </main>
    </div>

    <!-- 底部 Windows Terminal 风格控制台 (支持顶部拖拽调高 & 文本鼠标复制) -->
    <footer
      class="fluent-console"
      :style="{ height: consoleHeight + 'px' }"
    >
      <!-- 顶部拖拽把手 -->
      <div
        class="console-resizer"
        @mousedown="startConsoleResize"
        :title="t.chrome.console_resize_title"
      ></div>

      <div class="console-header">
        <div class="console-title-area">
          <span class="console-title">{{ t.console.title }}</span>
          <span class="log-count">({{ logs.length }})</span>
        </div>

        <div class="console-actions">
          <button class="console-btn" @click="clearLogs" :title="t.console.btn_clear">
            🧹 {{ t.console.btn_clear }}
          </button>
          <button class="console-btn" @click="copyLogs" :title="t.console.btn_copy">
            📋 {{ t.console.btn_copy }}
          </button>
        </div>
      </div>

      <!-- 控制台日志内容区 (支持鼠标划选、复制与独立滚动) -->
      <div class="console-body" ref="consoleBodyRef">
        <div v-if="logs.length === 0" class="console-empty">
          {{ t.console.empty }}
        </div>
        <div
          v-for="log in logs"
          :key="log.id"
          :class="['console-line', `log-${log.level}`]"
        >
          <span class="log-time">[{{ log.timestamp }}]</span>
          <span class="log-tag">[{{ log.level.toUpperCase() }}]</span>
          <span class="log-msg">{{ log.message }}</span>
        </div>
      </div>
    </footer>

    <!-- Win11 确认删除隧道模态弹窗 -->
    <div v-if="showDeleteModal" class="fluent-modal-overlay" @click.self="cancelDelete">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">
            ⚠️ {{ t.server_tab.errors.delete_confirm_title }}
          </h3>
        </div>
        <div class="modal-body">
          <p>{{ deleteConfirmMessage }}</p>
          <!-- 连带删除范围：该隧道绑定的域名（后端会一并删 CNAME）与其上的密码锁 -->
          <p v-if="deleteCascadeHint" class="modal-danger-hint">{{ deleteCascadeHint }}</p>
          <!-- 删除一律走 cloudflared tunnel delete -f：不管有没有服务在跑 / 还有没有活动连接 -->
          <p class="modal-danger-hint">{{ t.server_tab.delete_force_hint }}</p>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="cancelDelete">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn danger" @click="confirmDeleteTunnel">{{ t.server_tab.btn_delete }}</button>
        </div>
      </div>
    </div>

    <!-- DNS 绑定域名：改名弹窗（只改 DNS 记录名，不动隧道 ingress） -->
    <!-- DNS 路由绑定：添加绑定弹窗（隧道下拉 + 域名输入） -->
    <div v-if="showDnsAddModal" class="fluent-modal-overlay">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">🏷️ {{ t.server_tab.dns_add_title }}</h3>
        </div>
        <div class="modal-body">
          <div class="fluent-form-group">
            <label class="form-label">
              {{ t.server_tab.dns_col_tunnel }}
              <span class="required">*</span>
            </label>
            <div class="input-container">
              <!-- 固定域名列表为空时不渲染空的 select（会出现一个空输入格子），
                   改为一行提示；正常路径由 openDnsAddModal 提前拦截。 -->
              <select
                v-if="serverTunnelList.length > 0"
                v-model="dnsRoute.name"
                class="fluent-input fluent-select"
              >
                <option
                  v-for="tn in serverTunnelList"
                  :key="tn.id"
                  :value="tn.name"
                >{{ tn.name }}</option>
              </select>
              <div v-else class="address-hint">{{ t.server_tab.quick_list_empty }}</div>
            </div>
          </div>

          <div class="fluent-form-group">
            <label class="form-label">
              {{ t.server_tab.dns_domain }}
              <span class="required">*</span>
            </label>
            <div class="input-container">
              <input
                type="text"
                v-model="dnsRoute.domain"
                :placeholder="t.server_tab.dns_domain_placeholder"
                :class="['fluent-input', { 'input-error': dnsRouteDomainHasError }]"
                @input="onDnsRouteDomainInput"
                @keydown.enter="confirmDnsAdd"
              />
            </div>
            <div v-if="dnsRouteDomainHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.server_tab.errors.dns_domain_invalid }}
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="cancelDnsAdd">{{ t.exit_modal.btn_cancel }}</button>
          <button
            class="fluent-btn primary"
            @click="confirmDnsAdd"
            :disabled="!dnsRoute.name || !dnsRoute.domain || isDnsMutating"
          >
            {{ t.server_tab.btn_route_dns }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showDnsEditModal" class="fluent-modal-overlay" @click.self="cancelEditDnsRoute">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">✎ {{ t.server_tab.dns_edit_title }}</h3>
        </div>
        <div class="modal-body">
          <p class="modal-context">{{ dnsEditTarget?.tunnelName }} · {{ dnsEditTarget?.hostname }}</p>
          <div class="fluent-form-group">
            <label class="form-label">{{ t.server_tab.dns_edit_label }}</label>
            <div class="input-container">
              <input
                type="text"
                v-model="dnsEditValue"
                :placeholder="t.server_tab.dns_domain_placeholder"
                :class="['fluent-input', { 'input-error': dnsEditHasError }]"
                @input="dnsEditHasError = false"
                @keydown.enter="confirmEditDnsRoute"
              />
            </div>
            <div v-if="dnsEditHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.server_tab.errors.dns_domain_invalid }}
            </div>
          </div>
          <!-- 该域名上过锁时提示：锁绑在域名上，改名等于换了个域名，旧锁会被删掉 -->
          <p
            v-if="dnsEditTarget && lockOf(dnsEditTarget.hostname)"
            class="modal-danger-hint"
          >{{ t.server_tab.dns_rename_lock_hint }}</p>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="cancelEditDnsRoute">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn primary" @click="confirmEditDnsRoute">{{ t.server_tab.btn_save }}</button>
        </div>
      </div>
    </div>

    <!-- DNS 绑定域名：解绑确认弹窗 -->
    <div v-if="showDnsUnbindModal" class="fluent-modal-overlay" @click.self="cancelUnbindDnsRoute">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">⚠️ {{ t.server_tab.errors.dns_unbind_confirm_title }}</h3>
        </div>
        <div class="modal-body">
          <p>{{ t.server_tab.errors.dns_unbind_confirm_msg.replace('{name}', dnsUnbindTarget?.hostname || '').replace('{tunnel}', dnsUnbindTarget?.tunnelName || '') }}</p>
          <!-- 该域名若上过锁，解绑会连带把它删掉（锁只能挂在域名上，域名没了锁就成孤儿） -->
          <p
            v-if="dnsUnbindTarget && lockOf(dnsUnbindTarget.hostname)"
            class="modal-danger-hint"
          >{{ t.server_tab.dns_unbind_lock_hint }}</p>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="cancelUnbindDnsRoute">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn danger" @click="confirmUnbindDnsRoute">{{ t.server_tab.btn_unbind }}</button>
        </div>
      </div>
    </div>

    <!-- 客户端：新增 / 编辑隧道配置弹窗（配置持久保存，是否启动由列表里的按钮控制） -->
    <div v-if="showClientAddModal" class="fluent-modal-overlay">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">
            {{ editingClientKey ? `✎ ${t.client_tab.edit_title}` : `🔗 ${t.client_tab.add_title}` }}
          </h3>
        </div>
        <div class="modal-body">
          <div class="fluent-form-group">
            <label class="form-label">
              {{ t.client_tab.domain }}
              <span class="required">*</span>
            </label>
            <div class="input-container">
              <input
                type="text"
                v-model="clientForm.domain"
                :placeholder="t.client_tab.domain_placeholder"
                :class="['fluent-input', { 'input-error': clientFormDomainHasError }]"
                @input="onClientFormDomainInput"
                @keydown.enter="confirmClientAdd"
              />
            </div>
            <div v-if="clientFormDomainHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.client_tab.errors.domain_invalid }}
            </div>
          </div>

          <div class="fluent-form-group">
            <label class="form-label">
              {{ t.client_tab.port }}
              <span class="required">*</span>
            </label>
            <div class="input-container">
              <input
                type="text"
                v-model="clientForm.port"
                :placeholder="t.client_tab.port_placeholder"
                :class="['fluent-input', { 'input-error': clientFormPortHasError }]"
                @input="onClientFormPortInput"
                @keydown.enter="confirmClientAdd"
              />
            </div>
            <div v-if="clientFormPortHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.client_tab.errors.port_invalid }}
            </div>
          </div>

          <!-- 访问凭据（可选）：目标隧道开了密码锁时才需要填，两者必须成对 -->
          <div class="fluent-form-group">
            <label class="form-label">{{ t.client_tab.token_id_label }}</label>
            <div class="input-container">
              <input
                type="text"
                v-model="clientForm.tokenId"
                :placeholder="t.client_tab.token_id_placeholder"
                class="fluent-input mono"
                @keydown.enter="confirmClientAdd"
              />
            </div>
          </div>
          <div class="fluent-form-group">
            <label class="form-label">{{ t.client_tab.token_secret_label }}</label>
            <div class="input-container">
              <input
                type="text"
                v-model="clientForm.tokenSecret"
                :placeholder="t.client_tab.token_secret_placeholder"
                :class="['fluent-input mono', { 'input-error': clientTokenHasError }]"
                @keydown.enter="confirmClientAdd"
              />
            </div>
            <div v-if="clientTokenHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.client_tab.errors.token_pair_invalid }}
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="cancelClientAdd">{{ t.exit_modal.btn_cancel }}</button>
          <button
            class="fluent-btn primary"
            @click="confirmClientAdd"
            :disabled="!canSubmitClientAdd"
          >
            {{ t.client_tab.btn_save }}
          </button>
        </div>
      </div>
    </div>

    <!-- 客户端：删除二次确认弹窗 -->
    <div v-if="showClientDeleteModal" class="fluent-modal-overlay" @click.self="cancelDeleteClient">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">⚠️ {{ t.client_tab.delete_confirm_title }}</h3>
        </div>
        <div class="modal-body">
          <p>{{ t.client_tab.delete_confirm_msg.replace('{target}', `${pendingDeleteClient?.domain || ''}:${pendingDeleteClient?.port || ''}`) }}</p>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="cancelDeleteClient">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn danger" @click="confirmDeleteClient">{{ t.client_tab.btn_delete }}</button>
        </div>
      </div>
    </div>

    <!-- 临时链接：停止二次确认弹窗 -->
    <div v-if="showQuickStopModal" class="fluent-modal-overlay" @click.self="cancelStopQuick">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">⚠️ {{ t.server_tab.errors.quick_stop_confirm_title }}</h3>
        </div>
        <div class="modal-body">
          <p v-if="quickStopTarget" class="modal-context">{{ quickTargetLabel(quickStopTarget) }}</p>
          <p v-if="quickStopTarget?.url" class="modal-context mono">{{ quickStopTarget.url }}</p>
          <p>{{ t.server_tab.errors.quick_stop_confirm_msg }}</p>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="cancelStopQuick">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn danger" @click="confirmStopQuick">{{ t.server_tab.quick_stop }}</button>
        </div>
      </div>
    </div>

    <!-- 固定隧道：创建与修改**共用同一个弹窗**。
         固定隧道的全部内容都住在云端 ingress 里，「创建一条隧道」和「改一条隧道」要填的
         是同一批东西（协议 / 本地端口 / 域名），拆成两个界面只会让两边慢慢长歪。
         唯一的区别是标题、隧道名可编辑性，以及修改态要先读云端已有配置把表单填上。 -->
    <div v-if="showTunnelModal" class="fluent-modal-overlay">
      <div class="fluent-modal-dialog tunnel-form-dialog">
        <div class="modal-header">
          <h3 class="modal-title">
            {{ tunnelFormMode === 'create' ? t.server_tab.named_create_title : t.server_tab.named_edit_title }}
          </h3>
        </div>
        <div class="modal-body tunnel-form-body">
          <div v-if="tunnelFormLoading" class="route-loading">
            ⏳ {{ t.server_tab.form_loading_config }}
          </div>

          <!-- 隧道名字：创建时填写；修改时只读 —— 隧道名就是凭据文件名，改名要连带搬凭据、
               删旧锁，代价远大于收益，不如干脆不给改。 -->
          <div class="fluent-form-group">
            <label class="form-label">
              {{ t.server_tab.tunnel_name }}
              <span v-if="tunnelFormMode === 'create'" class="required">*</span>
            </label>
            <div class="input-container">
              <input
                v-if="tunnelFormMode === 'create'"
                type="text"
                v-model="tunnelFormName"
                :placeholder="t.server_tab.tunnel_name_placeholder"
                :class="['fluent-input', { 'input-error': tunnelFormNameHasError }]"
                @input="tunnelFormNameHasError = tunnelFormName.length > 0 && !isTunnelNameValid(tunnelFormName)"
                @keydown.enter="confirmTunnelForm"
              />
              <div v-else class="tunnel-form-name mono">{{ tunnelFormName }}</div>
            </div>
            <div v-if="tunnelFormMode === 'create' && tunnelFormNameHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.server_tab.errors.tunnel_invalid }}
            </div>
            <div v-else-if="tunnelFormMode === 'edit'" class="form-hint">
              {{ t.server_tab.form_name_readonly_hint }}
            </div>
          </div>

          <!-- ① 已发布应用程序路由 = 云端 ingress。
               一行 = 一条规则，用户只「选协议 + 填端口 + 填域名」，service 字符串由程序拼
               （http://127.0.0.1:8080 这种手写太容易错，也读不出来是哪台机器哪个服务）。 -->
          <div class="route-section">
            <div class="route-section-head">
              <span class="route-section-title">{{ t.server_tab.config_sec_published }}</span>
              <button class="fluent-btn small" @click="addIngressRow">
                <span class="btn-icon">＋</span>{{ t.server_tab.form_add_route }}
              </button>
            </div>

            <template v-for="(row, i) in tunnelFormRows" :key="'ing-' + i">
              <div
                class="ingress-row"
                :class="{ 'is-catch-all': isCatchAllIndex(i) }"
              >
                <div class="ingress-row-line">
                  <div class="ingress-field protocol">
                    <span class="ingress-field-label">
                      <!-- 末尾兜底行：它不靠域名匹配（是 ingress 的结构性末条）。标签跟「协议」
                           并排放在同一格的标签行里，整块因此只占一行高 -->
                      <span v-if="isCatchAllIndex(i)" class="catch-all-tag">{{ t.server_tab.form_catch_all_label }}</span>
                      <span>{{ t.server_tab.protocol_label }}</span>
                    </span>
                    <select
                      v-model="row.protocol"
                      class="fluent-input fluent-select"
                      @change="onIngressProtocolChange(row)"
                    >
                      <!-- 兜底行的默认档：未匹配就回 404。普通行不给这一项 —— 带域名回 404 没意义。
                           第二个条件是给存量数据留的：极少数老配置会给域名配 http_status:404，
                           那时它已是这一档，下拉得能选中它，否则界面显示会跟真实值对不上 -->
                      <option
                        v-if="isCatchAllIndex(i) || row.protocol === 'http_status_404'"
                        value="http_status_404"
                      >
                        {{ t.server_tab.protocol_catch_all_404 }}
                      </option>
                      <option value="http">{{ t.server_tab.protocol_http }}</option>
                      <option value="https">{{ t.server_tab.protocol_https }}</option>
                      <option value="tcp">{{ t.server_tab.protocol_tcp }}</option>
                      <option value="ssh">{{ t.server_tab.protocol_ssh }}</option>
                      <option value="rdp">{{ t.server_tab.protocol_rdp }}</option>
                      <option value="smb">{{ t.server_tab.protocol_smb }}</option>
                      <option value="unix">{{ t.server_tab.protocol_unix }}</option>
                      <option value="unix+tls">{{ t.server_tab.protocol_unix_tls }}</option>
                      <option value="hello_world">{{ t.server_tab.protocol_hello_world }}</option>
                      <!-- 反解不出协议+端口的存量规则：保留原样，别把云端已有的配置改掉 -->
                      <option v-if="row.protocol === 'raw'" value="raw">{{ t.server_tab.protocol_raw }}</option>
                    </select>
                  </div>

                  <div
                    v-if="row.protocol !== 'raw' && addressModeOf(row.protocol) !== 'none'"
                    class="ingress-field port"
                  >
                    <span class="ingress-field-label">
                      {{ addressModeOf(row.protocol) === 'socket' ? t.server_tab.unix_socket_label : t.server_tab.port }}
                    </span>
                    <input
                      v-if="addressModeOf(row.protocol) === 'port'"
                      type="text"
                      v-model="row.port"
                      :placeholder="t.server_tab.port_placeholder"
                      class="fluent-input"
                    />
                    <input
                      v-else
                      type="text"
                      v-model="row.unixSocket"
                      :placeholder="t.server_tab.unix_socket_placeholder"
                      class="fluent-input"
                    />
                  </div>

                  <div v-if="row.protocol === 'raw'" class="ingress-field service">
                    <span class="ingress-field-label">{{ t.server_tab.form_service_label }}</span>
                    <input type="text" v-model="row.rawService" class="fluent-input mono" />
                  </div>
                  <!-- 兜底行没有域名输入框：它按定义就不带域名 -->
                  <div v-else-if="!isCatchAllIndex(i)" class="ingress-field hostname">
                    <!-- 域名空着就是空着：不报错、不改变这条的含义，保存时跳过它（不写入云端）。
                         要接住所有未匹配的请求，用末尾那条常驻的「默认兜底」 -->
                    <span class="ingress-field-label">{{ t.server_tab.form_hostname_label }}</span>
                    <input
                      type="text"
                      v-model="row.hostname"
                      :placeholder="t.server_tab.form_route_hostname_placeholder"
                      class="fluent-input"
                    />
                  </div>

                  <!-- 兜底行：说明跟协议下拉同排（占掉剩余宽度），并把最终会写进云端的
                       service 原文亮出来；整块因此只有一行高。放不下时省略号收尾，
                       完整原文挂 title 上，鼠标一悬停就能看到 -->
                  <div
                    v-if="isCatchAllIndex(i)"
                    class="catch-all-hint"
                    :title="t.server_tab.form_catch_all_hint + ' ' + serviceOfRow(row)"
                  >
                    {{ t.server_tab.form_catch_all_hint }}
                    <span v-if="!ingressRowErrorKey(row, i)" class="catch-all-service mono">
                      {{ serviceOfRow(row) }}
                    </span>
                  </div>

                  <!-- 兜底行不可删：ingress 最后一条必须不带域名，删了得上哪找一条 -->
                  <button
                    v-if="!isCatchAllIndex(i)"
                    class="row-action-btn danger ingress-remove"
                    :title="t.server_tab.form_remove_route"
                    @click="removeIngressRow(i)"
                  >🗑</button>
                </div>
                <div v-if="tunnelFormSubmitted && ingressRowErrorKey(row, i)" class="error-tip">
                  <span class="error-icon">⚠️</span>
                  {{ errText(ingressRowErrorKey(row, i)) }}
                </div>
              </div>
            </template>
          </div>

          <!-- ② 主机名路由（WARP 私网访问，一般留空） -->
          <div class="route-section">
            <div class="route-section-head">
              <span class="route-section-title">{{ t.server_tab.config_sec_hostname }}</span>
              <button class="fluent-btn small" @click="addHostRoute">
                <span class="btn-icon">＋</span>{{ t.server_tab.form_add_host_route }}
              </button>
            </div>
            <div v-if="tunnelFormHostRoutes.length === 0" class="route-empty">
              {{ t.server_tab.form_host_routes_hint }}
            </div>
            <div v-for="(r, i) in tunnelFormHostRoutes" :key="'host-' + i" class="route-row">
              <input
                type="text"
                v-model="r.hostname"
                :placeholder="t.server_tab.form_host_label"
                class="fluent-input mono"
              />
              <input
                type="text"
                v-model="r.comment"
                :placeholder="t.server_tab.form_comment_label"
                class="fluent-input"
              />
              <button class="row-action-btn danger" @click="tunnelFormHostRoutes.splice(i, 1)">🗑</button>
            </div>
            <div v-if="tunnelFormSubmitted && tunnelFormHostRoutes.some(r => !r.hostname.trim())" class="error-tip">
              <span class="error-icon">⚠️</span>{{ errText('err_host_required') }}
            </div>
          </div>

          <!-- ③ CIDR 路由（WARP 私网访问，一般留空） -->
          <div class="route-section">
            <div class="route-section-head">
              <span class="route-section-title">{{ t.server_tab.config_sec_cidr }}</span>
              <button class="fluent-btn small" @click="addCidrRoute">
                <span class="btn-icon">＋</span>{{ t.server_tab.form_add_cidr_route }}
              </button>
            </div>
            <div v-if="tunnelFormCidrRoutes.length === 0" class="route-empty">
              {{ t.server_tab.form_cidr_routes_hint }}
            </div>
            <div v-for="(r, i) in tunnelFormCidrRoutes" :key="'cidr-' + i" class="route-row">
              <input
                type="text"
                v-model="r.network"
                :placeholder="t.server_tab.form_cidr_label"
                class="fluent-input mono"
              />
              <input
                type="text"
                v-model="r.comment"
                :placeholder="t.server_tab.form_comment_label"
                class="fluent-input"
              />
              <button class="row-action-btn danger" @click="tunnelFormCidrRoutes.splice(i, 1)">🗑</button>
            </div>
            <div v-if="tunnelFormSubmitted && tunnelFormCidrRoutes.some(r => !r.network.trim())" class="error-tip">
              <span class="error-icon">⚠️</span>{{ errText('err_cidr_required') }}
            </div>
          </div>

          <div v-if="tunnelFormLoadError" class="modal-hint warn">
            {{ t.server_tab.form_load_failed }}: {{ tunnelFormLoadError }}
          </div>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="showTunnelModal = false">{{ t.exit_modal.btn_cancel }}</button>
          <button
            class="fluent-btn primary"
            @click="confirmTunnelForm"
            :disabled="tunnelFormSaving || tunnelFormLoading"
          >
            {{ tunnelFormMode === 'create' ? t.server_tab.btn_create : t.server_tab.btn_save }}
          </button>
        </div>
      </div>
    </div>

    <!-- 密码锁：上锁 / 换密码成功后的凭据展示（明文 + 点击复制） -->
    <div v-if="showLockInfoModal" class="fluent-modal-overlay">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">🔐 {{ t.server_tab.lock_success_title }}</h3>
        </div>
        <div class="modal-body">
          <p class="modal-hint">{{ t.server_tab.lock_success_msg }}</p>
          <div class="fluent-form-group">
            <label class="form-label">{{ t.server_tab.lock_host_label }}</label>
            <div class="input-container">
              <input type="text" :value="lockInfoDraft?.hostname" class="fluent-input mono" readonly />
            </div>
          </div>
          <div class="fluent-form-group">
            <label class="form-label">{{ t.server_tab.lock_account_label }}</label>
            <div class="input-container">
              <input
                type="text"
                :value="lockInfoDraft?.clientId"
                class="fluent-input mono click-copy"
                readonly
                :title="t.server_tab.click_to_copy"
                @click="copyText(lockInfoDraft?.clientId || '')"
              />
            </div>
          </div>
          <div class="fluent-form-group">
            <label class="form-label">{{ t.server_tab.lock_secret_label }}</label>
            <div class="input-container">
              <input
                type="text"
                :value="lockInfoDraft?.clientSecret"
                class="fluent-input mono click-copy"
                readonly
                :title="t.server_tab.click_to_copy"
                @click="copyText(lockInfoDraft?.clientSecret || '')"
              />
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn primary" @click="showLockInfoModal = false">{{ t.server_tab.lock_done_btn }}</button>
        </div>
      </div>
    </div>

    <!-- 密码锁：解锁二次确认弹窗 -->
    <div v-if="showUnlockModal" class="fluent-modal-overlay" @click.self="showUnlockModal = false">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">⚠️ {{ t.server_tab.unlock_confirm_title }}</h3>
        </div>
        <div class="modal-body">
          <p>{{ t.server_tab.unlock_confirm_msg.replace('{hostname}', unlockTarget || '') }}</p>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="showUnlockModal = false">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn danger" @click="confirmUnlockHostname" :disabled="isLockMutating">
            {{ t.server_tab.btn_unlock }}
          </button>
        </div>
      </div>
    </div>

    <!-- 密码锁：换密码二次确认弹窗（旧密码会立即作废） -->
    <div v-if="showRotateModal" class="fluent-modal-overlay" @click.self="showRotateModal = false">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">🔁 {{ t.server_tab.rotate_confirm_title }}</h3>
        </div>
        <div class="modal-body">
          <p>{{ t.server_tab.rotate_confirm_msg.replace('{hostname}', rotateTarget || '') }}</p>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="showRotateModal = false">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn primary" @click="confirmRotatePassword" :disabled="isLockMutating">
            {{ t.server_tab.btn_rotate_password }}
          </button>
        </div>
      </div>
    </div>

    <!-- 临时链接：创建弹窗（协议 + 端口/套接字，域名由 Cloudflare 随机分配） -->
    <div v-if="showQuickCreateModal" class="fluent-modal-overlay">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">⚡ {{ t.server_tab.quick_create_title }}</h3>
        </div>
        <div class="modal-body">
          <div class="fluent-form-group">
            <label class="form-label">{{ t.server_tab.protocol_label }}</label>
            <div class="input-container">
              <select v-model="quickConfig.protocol" class="fluent-input fluent-select">
                <option value="http">{{ t.server_tab.protocol_http }}</option>
                <option value="https">{{ t.server_tab.protocol_https }}</option>
                <option value="tcp">{{ t.server_tab.protocol_tcp }}</option>
                <option value="ssh">{{ t.server_tab.protocol_ssh }}</option>
                <option value="rdp">{{ t.server_tab.protocol_rdp }}</option>
                <option value="smb">{{ t.server_tab.protocol_smb }}</option>
                <option value="unix">{{ t.server_tab.protocol_unix }}</option>
                <option value="unix+tls">{{ t.server_tab.protocol_unix_tls }}</option>
                <option value="hello_world">{{ t.server_tab.protocol_hello_world }}</option>
              </select>
            </div>
          </div>

          <div v-if="quickAddressMode !== 'none'" class="fluent-form-group">
            <label class="form-label">
              {{ quickAddressMode === 'socket' ? t.server_tab.unix_socket_label : t.server_tab.quick_port_label }}
              <span class="required">*</span>
            </label>
            <div class="input-container">
              <input
                v-if="quickAddressMode === 'port'"
                type="text"
                v-model="quickConfig.port"
                :placeholder="t.server_tab.quick_port_placeholder"
                :class="['fluent-input', { 'input-error': quickPortHasError }]"
                @input="onQuickPortInput"
                @keydown.enter="confirmQuickCreate"
              />
              <input
                v-else
                type="text"
                v-model="quickConfig.unixSocket"
                :placeholder="t.server_tab.unix_socket_placeholder"
                class="fluent-input"
                @input="onQuickUnixSocketInput"
                @keydown.enter="confirmQuickCreate"
              />
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="showQuickCreateModal = false">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn primary" @click="confirmQuickCreate" :disabled="!canStartQuick">
            {{ t.server_tab.btn_generate_quick }}
          </button>
        </div>
      </div>
    </div>

    <!-- Win11 退出应用二次确认模态弹窗 -->
    <div v-if="showExitConfirmModal" class="fluent-modal-overlay" @click.self="showExitConfirmModal = false">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">🚪 {{ t.exit_modal.title }}</h3>
        </div>
        <div class="modal-body">
          <p>{{ t.exit_modal.message }}</p>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="showExitConfirmModal = false">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn danger" @click="confirmExitApp">{{ t.exit_modal.btn_confirm }}</button>
        </div>
      </div>
    </div>

    <!-- Win11 Toast 消息提示 -->
    <transition name="toast-fade">
      <div v-if="toastMessage" class="fluent-toast">
        <span class="toast-icon">✨</span>
        <span class="toast-text">{{ toastMessage }}</span>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { safeInvoke as invoke, safeListen as listen } from './utils/tauriBridge';
import { LANG_DATA, fmt } from './i18n';
import { TunnelInfo, QuickTunnelItem, ClientTunnelItem, LogEntry, DnsBinding } from './types';
import { isTunnelNameValid, isPortValid, isDomainValid } from './utils/validation';
import { getCloudflaredTarget } from './utils/cloudflaredDownloader';
import { soundManager } from './utils/sound';
import { playRandomEasterEggSound } from './utils/easterEggAudios';

// 窗口状态与控制
const isMaximized = ref(false);

const handleMinimize = async () => {
  try {
    await invoke('minimize_window');
  } catch (e) {
    console.error('Minimize error:', e);
  }
};

const handleToggleMaximize = async () => {
  try {
    const isMax = await invoke<boolean>('toggle_maximize_window');
    isMaximized.value = isMax;
  } catch (e) {
    console.error('Toggle maximize error:', e);
  }
};

const handleCloseWindow = async () => {
  try {
    await invoke('close_window');
  } catch (e) {
    console.error('Close/Hide window error:', e);
  }
};

// 主题状态与动画
const isDarkMode = ref(localStorage.getItem('app_theme') !== 'light');
const isThemeSpinning = ref(false);

// 音效状态与动画
const isSoundEnabled = ref(localStorage.getItem('app_sound_enabled') !== 'false');
const isSoundSpinning = ref(false);
soundManager.setEnabled(isSoundEnabled.value);

// Logo 彩蛋状态
const isLogoShaking = ref(false);

const onLogoClick = () => {
  isLogoShaking.value = true;
  playRandomEasterEggSound(isSoundEnabled.value);
  setTimeout(() => {
    isLogoShaking.value = false;
  }, 380);
};

// 切换全局音效开关
const toggleSound = () => {
  isSoundSpinning.value = true;
  isSoundEnabled.value = !isSoundEnabled.value;
  soundManager.setEnabled(isSoundEnabled.value);
  localStorage.setItem('app_sound_enabled', isSoundEnabled.value ? 'true' : 'false');
  if (isSoundEnabled.value) {
    soundManager.playSuccess();
    showToast(t.value.logs.toast_sound_on);
  } else {
    showToast(t.value.logs.toast_sound_off);
  }
  setTimeout(() => {
    isSoundSpinning.value = false;
  }, 400);
};

// 同步主题至 html 和 body 标签，彻底根除深色模式下的白边
const syncThemeToDocument = () => {
  if (isDarkMode.value) {
    document.documentElement.classList.add('dark-theme');
    document.documentElement.classList.remove('light-theme');
    document.body.classList.add('dark-theme');
    document.body.classList.remove('light-theme');
  } else {
    document.documentElement.classList.add('light-theme');
    document.documentElement.classList.remove('dark-theme');
    document.body.classList.add('light-theme');
    document.body.classList.remove('dark-theme');
  }
};

// 文案包只有一份（简体中文），但保留 computed 这层：脚本里 130 多处日志与吐司
// 写的是 t.value.<tab>.<key>，模板里也有大量 t.<tab>.<key>，包成 ref 就一行都不用改
const t = computed(() => LANG_DATA.zh_CN);

// 选项卡状态（点击软件默认进入「配置」页）
const currentTab = ref('misc');

// 侧边栏状态：整体默认折叠（图标栏），服务端下边栏默认折叠
const sidebarCollapsed = ref(true);
const sidebarOpen = ref<Record<string, boolean>>({
  server: false,
  client: false,
  misc: false,
});

// 点击侧边栏一级项：服务端需要连带处理下边栏的展开/折叠
const handleSidebarClick = (tab: string) => {
  if (tab === 'server') {
    if (sidebarCollapsed.value) {
      // 折叠态下点击：先展开侧边栏，同时展开服务端下边栏
      sidebarCollapsed.value = false;
      sidebarOpen.value.server = true;
    } else {
      // 展开态下点击：切换下边栏的展开/折叠
      sidebarOpen.value.server = !sidebarOpen.value.server;
    }
  }
  switchTab(tab);
};

// 服务端下边栏的二视图切换：临时隧道 / 固定隧道
// （原「云端托管」入口已取消：托管模式是隧道的属性，不是另一个列表）
const switchServerView = (view: 'quick' | 'named') => {
  switchLocalSubMode(view);
  sidebarOpen.value.server = true;
  switchTab('server');
};

// 当前是否处于服务端某个子视图
const isServerView = (view: 'quick' | 'named') => localSubMode.value === view;

// 表单输入
const serverConfig = ref({
  name: localStorage.getItem('server_tunnel_name') || 'mc',
  port: localStorage.getItem('server_port') || '25565',
  protocol: localStorage.getItem('server_protocol') || 'http',
  unixSocket: localStorage.getItem('server_unix_socket') || '',
});

const clientForm = ref({
  domain: '',
  port: localStorage.getItem('client_port') || '25566',
  // 访问凭据（可选）：目标隧道开了密码锁时才需要，成对填写
  tokenId: '',
  tokenSecret: '',
});

// DNS 路由绑定表单 (cloudflared tunnel route dns)
const dnsRoute = ref({
  name: localStorage.getItem('dns_route_name') || serverConfig.value.name || '',
  domain: localStorage.getItem('dns_route_domain') || '',
});

// 输入错误校验状态（服务端的隧道名校验已并入统一弹窗：tunnelFormNameHasError）
const clientFormDomainHasError = ref(false);
const clientFormPortHasError = ref(false);
const dnsRouteNameHasError = ref(false);
const dnsRouteDomainHasError = ref(false);

// DNS 绑定域名管理（改名 / 解绑），只作用于固定域名的绑定记录
type DnsBoundRow = {
  recordId: string;
  hostname: string;
  tunnelName: string;
  tunnelId: string;
};
const showDnsEditModal = ref(false);
const showDnsUnbindModal = ref(false);
const showDnsAddModal = ref(false);
const dnsEditTarget = ref<DnsBoundRow | null>(null);
const dnsUnbindTarget = ref<DnsBoundRow | null>(null);
const dnsEditValue = ref('');
const dnsEditHasError = ref(false);
// 改名 / 解绑进行中，避免重复提交
const isDnsMutating = ref(false);

// 运行状态：命名隧道支持多开，后端 is_server_running 返回正在运行的隧道名列表（key = 隧道名）
const serverRunningNames = ref<string[]>([]);
// 某条命名隧道是否正在运行
const isTunnelRunning = (name: string) => {
  const n = name.trim();
  return !!n && serverRunningNames.value.includes(n);
};
// 本地标记某条隧道为运行中 / 已停止
const markServerRunning = (name: string) => {
  const n = name.trim();
  if (!n || serverRunningNames.value.includes(n)) return;
  serverRunningNames.value = [...serverRunningNames.value, n];
};
const markServerStopped = (name: string) => {
  const n = name.trim();
  serverRunningNames.value = serverRunningNames.value.filter(x => x !== n);
};
// 与后端对账运行状态（后端用 try_wait() 清理已退出的进程）
const reconcileServerRunning = async () => {
  try {
    serverRunningNames.value = await invoke<string[]>('is_server_running');
  } catch {}
};

// 客户端隧道支持多开：配置持久保存（localStorage），运行状态与后端对账。
// 断开只是停掉 cloudflared 进程，条目留在列表里，随时可再次启动。
const clientConnections = ref<ClientTunnelItem[]>([]);
// tokenId / tokenSecret 是目标隧道的访问凭据（Service Token），开启密码锁的隧道必须携带
type SavedClientTunnel = { key: string; domain: string; port: string; tokenId?: string; tokenSecret?: string };
const CLIENT_TUNNELS_STORAGE_KEY = 'client_tunnels_v1';
// 分隔符必须与 Rust 侧 client_tunnel_key / 浏览器 mock 一致（域名|端口）。
// 不一致会导致「已保存的配置」与「后端在跑的实例」key 不同，对账时被判成两条不同隧道，
// 列表里就会同一域名端口出现两行（一行未连接、一行已连接）。
const clientTunnelKey = (domain: string, port: string) => `${domain.trim()}|${port.trim()}`;

const loadSavedClientTunnels = (): SavedClientTunnel[] => {
  try {
    const raw = localStorage.getItem(CLIENT_TUNNELS_STORAGE_KEY);
    if (!raw) return [];
    const arr = JSON.parse(raw);
    if (!Array.isArray(arr)) return [];
    // 重新计算 key 并按 key 去重：老版本分隔符不同，对账时可能已经补进过重复条目
    const out: SavedClientTunnel[] = [];
    const seen = new Set<string>();
    for (const x of arr) {
      if (!x || typeof x.domain !== 'string' || typeof x.port !== 'string') continue;
      const domain = String(x.domain).trim();
      const port = String(x.port).trim();
      const key = clientTunnelKey(domain, port);
      if (seen.has(key)) continue;
      seen.add(key);
      const entry: SavedClientTunnel = { key, domain, port };
      // 访问凭据可选：旧版本没有这两个字段，读出来是 undefined 就当没配
      if (typeof x.tokenId === 'string' && x.tokenId.trim()) entry.tokenId = x.tokenId.trim();
      if (typeof x.tokenSecret === 'string' && x.tokenSecret.trim()) entry.tokenSecret = x.tokenSecret.trim();
      out.push(entry);
    }
    return out;
  } catch {
    return [];
  }
};
const savedClientTunnels = ref<SavedClientTunnel[]>(loadSavedClientTunnels());
const persistClientTunnels = () => {
  localStorage.setItem(
    CLIENT_TUNNELS_STORAGE_KEY,
    JSON.stringify(savedClientTunnels.value.map(({ domain, port, tokenId, tokenSecret }) => ({ domain, port, tokenId, tokenSecret }))),
  );
};

// 后端此刻真正在跑的客户端隧道 key（一条隧道一个 cloudflared access 进程）
const clientRunningKeys = ref<string[]>([]);
const clientRunningCount = computed(() => clientRunningKeys.value.length);
const isClientRunning = (key: string) => clientRunningKeys.value.includes(key);

// 表格行 = 已保存的配置 ∪ 后端正在跑的实例（旧版本遗留或外部启动的也能看到）
const clientRows = computed(() => {
  const rows: Array<SavedClientTunnel & { running: boolean }> = savedClientTunnels.value.map(
    t => ({ ...t, running: isClientRunning(t.key) }),
  );
  const known = new Set(rows.map(r => r.key));
  for (const item of clientConnections.value) {
    if (!known.has(item.key)) {
      rows.push({ key: item.key, domain: item.domain, port: item.port, running: true });
      known.add(item.key);
    }
  }
  // 兜底去重：同一「域名|端口」无论来源如何，界面上只允许出现一行
  const unique = new Map<string, SavedClientTunnel & { running: boolean }>();
  for (const r of rows) unique.set(r.key, r);
  return [...unique.values()];
});

const showClientAddModal = ref(false);
// 删除二次确认弹窗：pendingDeleteClient 是等待确认的那一条
const showClientDeleteModal = ref(false);
const pendingDeleteClient = ref<SavedClientTunnel | null>(null);
// 非空表示弹窗处于「编辑」模式，值是正在编辑条目的 key
const editingClientKey = ref('');
// 提交中标记，确认按钮据此避免重复提交
const clientSubmitting = ref(false);
// 访问凭据必须成对：只填账号或只填密码都视为填写错误
const clientTokenHasError = computed(
  () => clientForm.value.tokenId.trim().length > 0 !== clientForm.value.tokenSecret.trim().length > 0,
);
const canSubmitClientAdd = computed(
  () =>
    !clientSubmitting.value &&
    !clientFormDomainHasError.value &&
    !clientFormPortHasError.value &&
    !clientTokenHasError.value &&
    !!clientForm.value.domain.trim() &&
    !!clientForm.value.port.trim()
);
// 刷新态按列表分开：固定隧道 / 临时隧道各一份，互不牵连
const refreshingTunnels = ref<{ local: boolean; quick: boolean }>({ local: false, quick: false });
const isDownloadingCloudflared = ref(false);

// 固定隧道二级模式：临时隧道 / 固定隧道
const localSubMode = ref<'quick' | 'named'>(localStorage.getItem('local_sub_mode') === 'quick' ? 'quick' : 'named');

const switchLocalSubMode = (mode: 'quick' | 'named') => {
  localSubMode.value = mode;
  localStorage.setItem('local_sub_mode', mode);
};

// 临时链接（临时域名）状态
const quickConfig = ref({
  port: localStorage.getItem('quick_port') || '5244',
  protocol: localStorage.getItem('quick_protocol') || 'http',
  unixSocket: localStorage.getItem('quick_unix_socket') || '',
});
const quickTunnels = ref<QuickTunnelItem[]>([]);
const quickRunning = computed(() => quickTunnels.value.length > 0);
const quickPortHasError = ref(false);

// 停止临时链接：二次确认弹窗（记录待停止的 key）
const showQuickStopModal = ref(false);
const quickStopKey = ref('');
const quickStopTarget = computed(
  () => quickTunnels.value.find(qt => qt.key === quickStopKey.value) || null
);

const onQuickPortInput = () => {
  const val = quickConfig.value.port;
  quickPortHasError.value = val.length > 0 && !isPortValid(val);
  localStorage.setItem('quick_port', val);
};

const onQuickUnixSocketInput = () => {
  localStorage.setItem('quick_unix_socket', quickConfig.value.unixSocket);
};

// 目标地址输入框形态，由协议决定：
//   port   —— 普通协议，填端口
//   socket —— unix / unix+tls，套接字路径直接顶替端口那一格
//   none   —— hello_world 内置测试服务器、兜底行的 404 档，不需要填写任何地址
type AddressMode = 'port' | 'socket' | 'none';
const addressModeOf = (protocol: string): AddressMode => {
  // 兜底行的默认档（http_status:404）：不接本机服务，直接回状态码，所以不需要地址
  if (protocol === CATCH_ALL_PROTOCOL) return 'none';
  if (protocol === 'hello_world') return 'none';
  if (protocol === 'unix' || protocol === 'unix+tls') return 'socket';
  return 'port';
};

const quickAddressMode = computed(() => addressModeOf(quickConfig.value.protocol));

const canStartQuick = computed(() => {
  const mode = quickAddressMode.value;
  if (mode === 'none') return true;
  if (mode === 'socket') return !!quickConfig.value.unixSocket.trim();
  return !quickPortHasError.value && !!quickConfig.value.port.trim();
});

// 解析后端返回的临时链接 key → 协议与端口/套接字
// key 格式：`协议://127.0.0.1:端口` / `unix:路径` / `unix+tls:路径` / `hello_world`
const parseQuickKey = (key: string): { protocol: string; port: string } => {
  const m = key.match(/^([a-z0-9]+):\/\/[^:]+:(\d+)$/i);
  if (m) return { protocol: m[1], port: m[2] };
  const u = key.match(/^(unix\+tls|unix):(.+)$/);
  if (u) return { protocol: u[1], port: u[2] };
  if (key === 'hello_world') return { protocol: 'hello_world', port: '' };
  return { protocol: 'http', port: '' };
};

// 临时链接目标描述：hello_world / unix / unix+tls 与普通协议显示格式不同
const quickTargetLabel = (qt: QuickTunnelItem) => {
  if (qt.protocol === 'hello_world') return t.value.logs.target_hello_world;
  if (qt.protocol === 'unix' || qt.protocol === 'unix+tls') return `${qt.protocol}:${qt.port}`;
  return `${qt.protocol}://127.0.0.1:${qt.port}`;
};

// ======================== 固定隧道表单（创建 / 修改共用） ========================
//
// 固定隧道的全部内容都住在云端 ingress 里，一条规则就是
//   { "hostname": "mc.example.com", "service": "tcp://127.0.0.1:25565" }
// 界面只让用户「选协议 + 填端口 + 填域名」，service 字符串由这里拼 ——
// 手写 service 既容易写错，也看不出到底是谁在监听哪个端口。
//
// 三块数据彼此独立、端点也各不相同，所以弹窗里分成三块编辑：
//   ① 已发布应用程序路由 → PUT .../cfd_tunnel/{id}/configurations 的 ingress
//   ② 主机名路由        → POST teamnet/routes/hostname，DELETE zerotrust/routes/hostname/{id}
//   ③ CIDR 路由         → POST / PATCH / DELETE teamnet/routes
type TunnelIngressRule = { hostname: string; path: string; service: string };
type TunnelHostnameRoute = { id: string; hostname: string; comment: string };
type TunnelCidrRoute = { id: string; network: string; comment: string };
type TunnelRouteSet = {
  hostname_routes: TunnelHostnameRoute[];
  cidr_routes: TunnelCidrRoute[];
  hostname_error: string | null;
  cidr_error: string | null;
};

type IngressRow = {
  /** 'raw' = 这条规则的 service 反解不出「协议 + 端口」，只能原文编辑 */
  protocol: string;
  port: string;
  unixSocket: string;
  /** 留空即「不带域名的规则」，它只能有一条且必须在最末（那条就是兜底） */
  hostname: string;
  /** protocol === 'raw' 时使用 */
  rawService: string;
  /** ingress 的 path 字段：界面不暴露，但读回来必须原样带回去 */
  path: string;
};

/** ingress 末尾的兜底规则：不带域名，接住所有未匹配的请求 */
const CATCH_ALL_SERVICE = 'http_status:404';
/** 兜底行下拉里的默认档。它不是真协议，serviceOfRow / rowOfRule / addressModeOf 都认它 */
const CATCH_ALL_PROTOCOL = 'http_status_404';

// 协议 → service 字符串。unix / hello_world 的写法跟普通协议不同，
// 与 Rust 侧 start_server_tunnel 拼 --url 的规则保持一字不差。
const serviceOfRow = (row: IngressRow): string => {
  if (row.protocol === CATCH_ALL_PROTOCOL) return CATCH_ALL_SERVICE;
  if (row.protocol === 'raw') return row.rawService.trim();
  if (row.protocol === 'hello_world') return 'hello_world';
  if (row.protocol === 'unix' || row.protocol === 'unix+tls') {
    return `${row.protocol}:${row.unixSocket.trim()}`;
  }
  return `${row.protocol}://127.0.0.1:${row.port.trim()}`;
};

// service → 表单行。反解不了就退化成 raw 行原文照存：宁可让用户看到一串看不懂的
// service，也不能把云端已有的规则悄悄改掉。
// host 部分同时接受 127.0.0.1 与 localhost —— 云端存量配置里两种都出现过。
const rowOfRule = (rule: TunnelIngressRule): IngressRow => {
  const base: IngressRow = {
    protocol: 'http',
    port: '',
    unixSocket: '',
    hostname: rule.hostname || '',
    rawService: '',
    path: rule.path || '',
  };
  const service = (rule.service || '').trim();
  // 兜底行的默认值：认出来就能在下拉里显示成「未匹配回 404」，
  // 而不是丢进 raw 让用户对着 http_status:404 发愣（改别的状态码仍落到 raw 原文保留）
  if (service === CATCH_ALL_SERVICE) return { ...base, protocol: CATCH_ALL_PROTOCOL };
  if (service === 'hello_world') return { ...base, protocol: 'hello_world' };
  const socket = service.match(/^(unix\+tls|unix):(.+)$/);
  if (socket) return { ...base, protocol: socket[1], unixSocket: socket[2] };
  const tcp = service.match(/^([a-z][a-z0-9]*):\/\/(?:127\.0\.0\.1|localhost):(\d+)$/i);
  if (tcp) return { ...base, protocol: tcp[1].toLowerCase(), port: tcp[2] };
  return { ...base, protocol: 'raw', rawService: service };
};

const emptyIngressRow = (): IngressRow => ({
  protocol: 'http',
  port: '',
  unixSocket: '',
  hostname: '',
  rawService: '',
  path: '',
});

// 兜底行的初始值：默认 http_status:404（未匹配就回 404）。
// 想让它把未匹配的请求转到本机某个服务，在下拉里换成具体协议 + 填端口即可。
const emptyCatchAllRow = (): IngressRow => ({
  ...emptyIngressRow(),
  protocol: CATCH_ALL_PROTOCOL,
});

// 弹窗状态：create 与 edit 共用同一套，靠 tunnelFormMode 分辨
const showTunnelModal = ref(false);
const tunnelFormMode = ref<'create' | 'edit'>('create');
const tunnelFormTarget = ref<TunnelInfo | null>(null);
const tunnelFormName = ref('');
const tunnelFormRows = ref<IngressRow[]>([]);
const tunnelFormHostRoutes = ref<TunnelHostnameRoute[]>([]);
const tunnelFormCidrRoutes = ref<TunnelCidrRoute[]>([]);
const tunnelFormLoading = ref(false);
const tunnelFormSaving = ref(false);
const tunnelFormSubmitted = ref(false);
const tunnelFormNameHasError = ref(false);
const tunnelFormLoadError = ref('');
// 编辑态读回来的原始路由：保存时靠这两个快照算出「删了哪些 / 改了哪些」
const originalHostRoutes = ref<Record<string, string>>({});
const originalCidrRoutes = ref<Record<string, { network: string; comment: string }>>({});

// 数组最后一条就是兜底行 —— ingress 的最后一条必须不带域名，由它接住所有未匹配的请求。
// 它**常驻**：永远渲染、永远在最后、永远写进云端；不因为上面的行填成什么样而消失或变形，
// 也不参与任何「行与行之间」的推导。界面上它有「默认兜底」标签、没有域名输入框、不能删，
// service 由用户在下拉里选（默认 http_status:404，也可以换成转发到本机某个端口）。
const isCatchAllIndex = (i: number): boolean => i === tunnelFormRows.value.length - 1;

// 这一行会不会被写进云端：兜底行恒为真；普通行得填了域名才算一条路由。
// 域名空着就是空着 —— 不报错、不改变这条或别的行的含义，保存时整行跳过（见 buildIngress）。
const isRowWritten = (row: IngressRow, i: number): boolean =>
  isCatchAllIndex(i) || !!row.hostname.trim();

// 逐行校验：返回错误文案键（空串 = 合法）。只有点过「保存」之后才显示，免得一打开满屏红。
const ingressRowErrorKey = (row: IngressRow, i: number): string => {
  // 不会被写出去的行（域名空着的普通行）根本不参与校验 —— 它不构成配置，就不该拦保存，
  // 更不该逼用户去填那个域名。空着就空着，跳过它就是了（保存时给一条日志说明）。
  if (!isRowWritten(row, i)) return '';
  if (row.protocol === CATCH_ALL_PROTOCOL) return ''; // 404 档没有地址可填，也不可能填错
  if (row.protocol === 'raw') return row.rawService.trim() ? '' : 'err_service_required';
  const mode = addressModeOf(row.protocol);
  if (mode === 'socket' && !row.unixSocket.trim()) return 'err_socket_required';
  if (mode === 'port' && !isPortValid(row.port.trim())) return 'err_port_required';
  // 兜底行没有域名输入框，也不参与域名校验
  if (isCatchAllIndex(i)) return '';
  const host = row.hostname.trim();
  return isDomainValid(host) ? '' : 'err_hostname_invalid';
};

// 语言包里 errors 是手写接口，没有索引签名 —— 动态键必须这样取，
// 否则模板里 errText(...) 会被 vue-tsc 判成隐式 any。
const errText = (key: string): string => {
  const map = t.value.server_tab.errors as unknown as Record<string, string>;
  return map[key] || key;
};

const addIngressRow = () => {
  // 新行沿用上一条**普通行**的协议与端口（同一个隧道下多域名指向同一服务是常见做法），
  // 只清域名；插在兜底行之前，保证兜底行永远待在最后
  const rows = tunnelFormRows.value;
  const prev = rows[rows.length - 2];
  rows.splice(Math.max(rows.length - 1, 0), 0, prev ? { ...prev, hostname: '' } : emptyIngressRow());
};

// 兜底行不可删（它在数组末尾，模板也不给它删除按钮），其余行随便删 ——
// 只剩兜底行也是合法的 ingress（一条不带域名的规则）
const removeIngressRow = (i: number) => {
  if (isCatchAllIndex(i)) return;
  tunnelFormRows.value.splice(i, 1);
};

// 换协议时清掉上一个协议留下的值：unix 的套接字路径带进 http 行毫无意义
const onIngressProtocolChange = (row: IngressRow) => {
  const mode = addressModeOf(row.protocol);
  if (mode === 'port') row.unixSocket = '';
  else if (mode === 'socket') row.port = '';
  else if (mode === 'none') {
    row.port = '';
    row.unixSocket = '';
  }
};

const addHostRoute = () => {
  tunnelFormHostRoutes.value.push({ id: '', hostname: '', comment: '' });
};

const addCidrRoute = () => {
  tunnelFormCidrRoutes.value.push({ id: '', network: '', comment: '' });
};

// 表单行 → 云端 ingress 数组。兜底行也在 rows 里，永远落在最后一条。
// 普通行没填域名 → 整行跳过：Cloudflare 只允许**一条**不带 hostname 的规则，而且必须在最后，
// 那条恒由兜底行担任。把没填域名的普通行也写上去会变成两条「匹配所有 hostname」的规则，
// 云端直接判非法（"the rules which follow it will never be triggered"）。
const buildIngress = (): Record<string, string>[] => {
  const rows = tunnelFormRows.value;
  const rules: Record<string, string>[] = [];
  rows.forEach((row, i) => {
    if (!isRowWritten(row, i)) return;
    const rule: Record<string, string> = { service: serviceOfRow(row) };
    // 最后一条（兜底行）不写 hostname：Cloudflare 要求数组最后一条不带域名
    if (!isCatchAllIndex(i)) rule.hostname = row.hostname.trim();
    // path 只有存量规则才有；空值不往请求体里塞，免得云端把它当成「匹配空路径」
    if (row.path) rule.path = row.path;
    rules.push(rule);
  });
  // 纯防御：兜底行不可删，所以这里够不着；ingress 也绝不允许是空数组
  if (rules.length === 0) rules.push({ service: CATCH_ALL_SERVICE });
  return rules;
};

// invoke 被拒绝时抛出来的既可能是字符串（Rust 侧 Result<_, String>），也可能是别的对象，
// 统一转成能直接显示的一行文本。
const errorText = (e: unknown): string =>
  typeof e === 'string' ? e : String((e as Error)?.message ?? e);

// 打开「创建固定隧道」：与修改是同一个弹窗，只是标题、隧道名可编辑性、以及不去读云端不同
const openTunnelCreateModal = () => {
  soundManager.playClick();
  tunnelFormMode.value = 'create';
  tunnelFormTarget.value = null;
  tunnelFormName.value = serverConfig.value.name || 'mc';
  // 一行普通行 + 末尾那条常驻的兜底行（创建时就把兜底行摆出来，而不是等保存时程序偷偷补
  // 一条 —— 它是什么、能不能改，用户得看得见）。
  // 兜底行直接预置成本机服务：它是唯一会被写成「不带域名」的规则，
  // 所以「没有域名、未匹配的请求都转到本机」这件事在这儿明明白白摆着，不用靠留空普通行的域名去暗示。
  const catchAll = emptyCatchAllRow();
  const cfgProto = serverConfig.value.protocol || '';
  const cfgPort = (serverConfig.value.port || '').trim();
  const cfgSocket = (serverConfig.value.unixSocket || '').trim();
  if (addressModeOf(cfgProto) === 'port' && cfgPort) {
    catchAll.protocol = cfgProto;
    catchAll.port = cfgPort;
  } else if (addressModeOf(cfgProto) === 'socket' && cfgSocket) {
    catchAll.protocol = cfgProto;
    catchAll.unixSocket = cfgSocket;
  }
  tunnelFormRows.value = [
    {
      ...emptyIngressRow(),
      protocol: serverConfig.value.protocol || 'http',
      port: serverConfig.value.port || '',
    },
    catchAll,
  ];
  tunnelFormHostRoutes.value = [];
  tunnelFormCidrRoutes.value = [];
  originalHostRoutes.value = {};
  originalCidrRoutes.value = {};
  tunnelFormSubmitted.value = false;
  tunnelFormNameHasError.value = false;
  tunnelFormLoadError.value = '';
  showTunnelModal.value = true;
};

// 打开「修改隧道」：先把云端三块都读回来填进表单。
// 读失败不挡保存 —— 读不到（隧道没有云端配置 / 权限不足）时给一张空表，
// 用户照样能填完写上去，那正好把隧道变成云端托管。
const openTunnelEditModal = async (tunnel: TunnelInfo) => {
  soundManager.playClick();
  tunnelFormMode.value = 'edit';
  tunnelFormTarget.value = tunnel;
  tunnelFormName.value = tunnel.name.trim();
  // 先摆一行普通行 + 兜底行占位，读回云端配置后再整体替换
  tunnelFormRows.value = [emptyIngressRow(), emptyCatchAllRow()];
  tunnelFormHostRoutes.value = [];
  tunnelFormCidrRoutes.value = [];
  originalHostRoutes.value = {};
  originalCidrRoutes.value = {};
  tunnelFormSubmitted.value = false;
  tunnelFormNameHasError.value = false;
  tunnelFormLoadError.value = '';
  tunnelFormLoading.value = true;
  showTunnelModal.value = true;

  try {
    const [cfgRes, routeRes] = await Promise.allSettled([
      invoke<{ rules: TunnelIngressRule[] }>('fetch_tunnel_config', { tunnelId: tunnel.id }),
      invoke<TunnelRouteSet>('fetch_tunnel_routes', { tunnelId: tunnel.id }),
    ]);

    if (cfgRes.status === 'fulfilled') {
      const rows = (cfgRes.value?.rules ?? []).map(rowOfRule);
      // 末尾那条不带域名的规则就是兜底行，原样留在最后（它的 service 是用户定的，别覆盖）；
      // 云端如果没有兜底（最后一条带域名），补一条默认的摆出来让用户自己改
      const last = rows[rows.length - 1];
      if (!last || last.hostname.trim()) rows.push(emptyCatchAllRow());
      // 云端一条规则都没有（或只有一条不带域名的）：补一行空普通行，别让表单只剩兜底行，
      // 跟创建时的初始形态保持一致
      if (rows.length === 1 && !rows[0].hostname.trim()) rows.unshift(emptyIngressRow());
      tunnelFormRows.value = rows;
    } else {
      tunnelFormLoadError.value = errorText(cfgRes.reason);
    }

    if (routeRes.status === 'fulfilled') {
      tunnelFormHostRoutes.value = routeRes.value?.hostname_routes ?? [];
      tunnelFormCidrRoutes.value = routeRes.value?.cidr_routes ?? [];
      originalHostRoutes.value = Object.fromEntries(
        tunnelFormHostRoutes.value.map(r => [r.id, r.hostname]),
      );
      originalCidrRoutes.value = Object.fromEntries(
        tunnelFormCidrRoutes.value.map(r => [r.id, { network: r.network, comment: r.comment }]),
      );
      // 某一块读失败（多为 token 缺 Cloudflare One Networks 权限）只提示，不挡另外两块
      if (!tunnelFormLoadError.value) {
        tunnelFormLoadError.value = routeRes.value?.hostname_error || routeRes.value?.cidr_error || '';
      }
    } else if (!tunnelFormLoadError.value) {
      tunnelFormLoadError.value = errorText(routeRes.reason);
    }
  } catch (err: any) {
    tunnelFormLoadError.value = errorText(err);
  } finally {
    tunnelFormLoading.value = false;
  }
};

// 主机名路由：按「原始名单 / 当前名单」的差集增删。
// 改名 = 删旧建新（这个资源没有 PATCH）。删除必须走后端的 zerotrust 路径，
// 旧 teamnet 前缀对 DELETE 回 405，报错文案还骗人说「认证方案不支持」。
const syncHostnameRoutes = async (tunnelId: string, name: string) => {
  const keep = new Set(tunnelFormHostRoutes.value.map(r => r.id).filter(Boolean));
  for (const id of Object.keys(originalHostRoutes.value)) {
    if (keep.has(id)) continue;
    try {
      await invoke<string>('delete_hostname_route', { routeId: id });
      appendLog(
        `[SUCCESS] ${fmt(t.value.logs.host_route_deleted, { route: originalHostRoutes.value[id], name })}`,
        'success',
        'server',
      );
    } catch (err: any) {
      appendLog(`[WARN] ${fmt(t.value.logs.host_route_delete_failed, { err: errorText(err) })}`, 'warn', 'server');
    }
  }
  for (const r of tunnelFormHostRoutes.value) {
    const host = r.hostname.trim();
    if (r.id && originalHostRoutes.value[r.id] === host) continue;
    if (r.id) {
      // 名字改过：先把旧的删掉，再按新名字建
      try {
        await invoke<string>('delete_hostname_route', { routeId: r.id });
      } catch (err: any) {
        appendLog(`[WARN] ${fmt(t.value.logs.host_route_delete_failed_named, { route: originalHostRoutes.value[r.id], err: errorText(err) })}`, 'warn', 'server');
        continue;
      }
    }
    try {
      const res = await invoke<string>('create_hostname_route', {
        tunnelId,
        hostname: host,
        comment: r.comment.trim(),
      });
      appendLog(`[SUCCESS] ${res}`, 'success', 'server');
    } catch (err: any) {
      appendLog(`[WARN] ${fmt(t.value.logs.host_route_create_failed, { route: host, err: errorText(err) })}`, 'warn', 'server');
    }
  }
};

// CIDR 路由：同上，但这个资源支持 PATCH，改网段/备注走更新而不是删了重建
const syncCidrRoutes = async (tunnelId: string, name: string) => {
  const keep = new Set(tunnelFormCidrRoutes.value.map(r => r.id).filter(Boolean));
  for (const id of Object.keys(originalCidrRoutes.value)) {
    if (keep.has(id)) continue;
    try {
      await invoke<string>('delete_cidr_route', { routeId: id });
      appendLog(
        `[SUCCESS] ${fmt(t.value.logs.cidr_route_deleted, { network: originalCidrRoutes.value[id].network, name })}`,
        'success',
        'server',
      );
    } catch (err: any) {
      appendLog(`[WARN] ${fmt(t.value.logs.cidr_route_delete_failed, { err: errorText(err) })}`, 'warn', 'server');
    }
  }
  for (const r of tunnelFormCidrRoutes.value) {
    const network = r.network.trim();
    const comment = r.comment.trim();
    const before = r.id ? originalCidrRoutes.value[r.id] : undefined;
    if (before && before.network === network && before.comment === comment) continue;
    try {
      if (before) {
        const res = await invoke<string>('update_cidr_route', { routeId: r.id, network, comment });
        appendLog(`[SUCCESS] ${res}`, 'success', 'server');
      } else {
        const res = await invoke<string>('create_cidr_route', { tunnelId, network, comment });
        appendLog(`[SUCCESS] ${res}`, 'success', 'server');
      }
    } catch (err: any) {
      appendLog(`[WARN] ${fmt(t.value.logs.cidr_route_save_failed, { network, err: errorText(err) })}`, 'warn', 'server');
    }
  }
};

// 保存（创建与修改共用）：写 ingress → 补 DNS 路由 → 同步两类路由。
// 云端配置是**运行时生效**的：运行中的隧道会自动同步，不需要重启进程。
const confirmTunnelForm = async () => {
  tunnelFormSubmitted.value = true;
  const isCreate = tunnelFormMode.value === 'create';
  const name = tunnelFormName.value.trim();

  if (isCreate && (!name || !isTunnelNameValid(name))) {
    tunnelFormNameHasError.value = true;
    appendLog(`[ERROR] ${t.value.server_tab.errors.tunnel_invalid}`, 'error', 'server');
    return;
  }
  // 兜底行不可删，所以 rows 至少有一条；这里只是兜住「数组真的空了」这种不可能的情况
  if (tunnelFormRows.value.length === 0) tunnelFormRows.value = [emptyCatchAllRow()];

  const badRow = tunnelFormRows.value.findIndex((row, i) => ingressRowErrorKey(row, i));
  if (badRow >= 0) {
    appendLog(`[ERROR] ${fmt(t.value.logs.route_invalid_at, { index: badRow + 1 })}`, 'error', 'server');
    return;
  }
  // 没填域名的普通行不会写进云端（见 buildIngress）—— 只在它确实填了东西时才说一声，
  // 免得白填了服务却不知道为什么没生效；光秃秃的空行就当没看见
  const droppedRows = tunnelFormRows.value
    .map((row, i) => ({ row, i }))
    .filter(({ row, i }) => !isRowWritten(row, i))
    .filter(({ row }) => !!(row.port.trim() || row.unixSocket.trim() || row.rawService.trim()))
    .map(({ i }) => i + 1);
  if (droppedRows.length) {
    appendLog(
      `[WARN] ${fmt(t.value.logs.blank_host_rows_skipped, { rows: droppedRows.join('、') })}`,
      'warn',
      'server',
    );
  }
  // 同一个域名出现两次：云端按顺序只认第一条，第二条永远不会命中，属于白写
  const hosts = tunnelFormRows.value.map(r => r.hostname.trim()).filter(Boolean);
  if (new Set(hosts).size !== hosts.length) {
    appendLog(`[ERROR] ${errText('err_hostname_dup')}`, 'error', 'server');
    showToast(errText('err_hostname_dup'));
    return;
  }
  if (tunnelFormHostRoutes.value.some(r => !r.hostname.trim())) {
    appendLog(`[ERROR] ${errText('err_host_required')}`, 'error', 'server');
    return;
  }
  if (tunnelFormCidrRoutes.value.some(r => !r.network.trim())) {
    appendLog(`[ERROR] ${errText('err_cidr_required')}`, 'error', 'server');
    return;
  }

  tunnelFormSaving.value = true;
  try {
    let tunnelId = tunnelFormTarget.value?.id || '';
    if (isCreate) {
      const res = await invoke<string>('create_tunnel', { name });
      appendLog(`[SUCCESS] ${fmt(t.value.logs.tunnel_created, { name, res })}`, 'success', 'server');
      // 新隧道要拿它的 ID 才能写云端配置，而 ID 只能从刷新后的列表里取
      await handleRefreshTunnels();
      // 注意别把这个 lambda 参数叫 t —— 会遮住 i18n 的 t，自检脚本也会误判成文案键
      tunnelId = tunnelList.value.find(tn => tn.name.trim() === name)?.id || '';
      if (!tunnelId) {
        throw new Error(t.value.logs.err_tunnel_id_missing);
      }
    }
    if (!tunnelId) throw new Error(t.value.logs.err_tunnel_id_not_found);

    // ① 已发布应用程序路由
    await invoke<string>('update_tunnel_config', { tunnelId, ingress: buildIngress() });
    appendLog(`[SUCCESS] ${fmt(t.value.logs.ingress_written, { name })}`, 'success', 'server');

    // ② 新增的域名补 DNS 路由。
    //    只补不删：删域名不连带删 DNS 记录，免得误删别处在用的 CNAME，
    //    要解绑请去「DNS 路由绑定」面板（那里会连带清掉该域名的密码锁）。
    const bound = new Set(
      (tunnelList.value.find(tn => tn.id === tunnelId)?.hostnames ?? []).map(h => h.name),
    );
    for (const host of hosts) {
      if (bound.has(host)) continue;
      try {
        const dnsRes = await invoke<string>('route_dns_tunnel', { name, hostname: host });
        appendLog(`[SUCCESS] ${dnsRes}`, 'success', 'server');
      } catch (err: any) {
        appendLog(`[WARN] ${fmt(t.value.logs.dns_route_create_failed, { host, err: errorText(err) })}`, 'warn', 'server');
      }
    }

    // ③ 主机名路由 / CIDR 路由
    await syncHostnameRoutes(tunnelId, name);
    await syncCidrRoutes(tunnelId, name);

    // ④ 记住源站配置：列表行内「启动」直接用。
    //    真正生效的是云端 ingress，这份本地记录只是为了让启动按钮不必先打网络请求。
    //    必须挑一条**真的写进云端了的转发行**：兜底行的 404 档是假协议，存进去启动按钮
    //    会当端口模式校验反而拦住启动；没填域名被跳过的行同理 —— 它压根没写进 ingress。
    const writtenRows = tunnelFormRows.value.filter((r, i) => isRowWritten(r, i));
    const first =
      writtenRows.find(r => r.protocol !== 'raw' && r.protocol !== CATCH_ALL_PROTOCOL) ??
      writtenRows.find(r => r.protocol !== CATCH_ALL_PROTOCOL) ??
      null;
    if (first) {
      saveTunnelCfgValues(name, first.protocol, first.port.trim(), first.unixSocket.trim());
      if (isCreate) serverConfig.value.name = name;
    }

    await handleRefreshTunnels();
    await refreshHostnamesOnly();
    soundManager.playSuccess();
    showToast(t.value.server_tab.form_saved);
    showTunnelModal.value = false;
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.tunnel_save_failed, { err: errorText(err) })}`, 'error', 'server');
    showToast(`${errorText(err)}`);
  } finally {
    tunnelFormSaving.value = false;
  }
};
// 隧道列表与选中项
const tunnelList = ref<TunnelInfo[]>([]);
const selectedTunnel = ref<TunnelInfo | null>(null);

// 账号下的**全部**隧道，只此一份。原先按 tunnel_type 拆成「固定域名 / 云端托管」两份，
// 但那个字段判的是「本机有没有凭据文件」，跟「是不是云端托管」是两回事 ——
// 于是同一条隧道会出现在两处，或者明明是云端托管却被标成「本地」。
const serverTunnelList = computed(() => tunnelList.value);

// 固定隧道列表的「运行中 (数量)」统计
const localRunningCount = computed(() =>
  serverTunnelList.value.filter(x => isTunnelRunning(x.name)).length,
);

// ============================ 隧道密码锁（Cloudflare Access） ============================
//
// 锁本质是绑在「域名」上的（Access 应用按域名建），所以锁记录按 hostname 存，
// 一个隧道绑多个域名时每个域名各自独立上锁 / 换密码 / 解锁。
// 上锁 = 后端在云端创建 Access 应用 + Service Token，拿到「访问账号 / 访问密码」。
type TunnelLockEntry = {
  hostname: string;
  appUid: string;
  appName: string;
  tokenUid: string;
  /** 访问策略 UID；老版本条目没有该字段（空串），解锁时后端会按 token_id 反查 */
  policyUid: string;
  clientId: string;
  clientSecret: string;
};
const TUNNEL_LOCKS_STORAGE_KEY = 'tunnel_locks_v1';

const loadTunnelLocks = (): Record<string, TunnelLockEntry> => {
  try {
    const raw = localStorage.getItem(TUNNEL_LOCKS_STORAGE_KEY);
    if (!raw) return {};
    const obj = JSON.parse(raw);
    if (!obj || typeof obj !== 'object' || Array.isArray(obj)) return {};
    // 只保留字段齐全的条目，坏数据直接丢弃（最坏情况是列表里显示未上锁，
    // 云端那把锁仍在，可在面板手动清理）。
    // 老版本条目按隧道 ID 存（e.tunnelId），本版改为按 hostname 存 —— 迁移时
    // 以 hostname 为键重建，键重复时后写覆盖，保证每个域名只有一条锁。
    const out: Record<string, TunnelLockEntry> = {};
    for (const v of Object.values(obj as Record<string, unknown>)) {
      const e = v as Partial<TunnelLockEntry> & { tunnelId?: string };
      if (
        typeof e?.hostname === 'string' && e.hostname &&
        typeof e?.appUid === 'string' && e.appUid &&
        typeof e?.tokenUid === 'string' && e.tokenUid &&
        typeof e?.clientId === 'string' && e.clientId &&
        typeof e?.clientSecret === 'string' && e.clientSecret
      ) {
        out[e.hostname] = {
          hostname: e.hostname,
          appUid: e.appUid,
          appName: String(e.appName || ''),
          tokenUid: e.tokenUid,
          policyUid: typeof e.policyUid === 'string' ? e.policyUid : '',
          clientId: e.clientId,
          clientSecret: e.clientSecret,
        };
      }
    }
    return out;
  } catch {
    return {};
  }
};

const tunnelLocks = ref<Record<string, TunnelLockEntry>>(loadTunnelLocks());
const persistTunnelLocks = () => {
  localStorage.setItem(TUNNEL_LOCKS_STORAGE_KEY, JSON.stringify(tunnelLocks.value));
};
// 按域名查锁：域名是锁的唯一载体，也是客户端连接时真正要锁住的目标
const lockOf = (hostname: string): TunnelLockEntry | null => tunnelLocks.value[hostname] || null;

// 锁操作进行中：所有锁按钮统一禁用，防止并发重复建锁
const isLockMutating = ref(false);

// 密码锁凭据默认掩码显示：client id 32 位 + secret 40 位，明文会把域名行撑破。
// 想核对全文时点「显示凭据」展开；展开状态按域名记，纯临时 UI 态，不落盘。
const expandedLockCreds = ref<Record<string, boolean>>({});
const isLockCredExpanded = (hostname: string): boolean => !!expandedLockCreds.value[hostname];
const toggleLockCred = (hostname: string) => {
  // 整体替换对象而不是改属性：record 里新增的 key 不是响应式的，直接赋值不会触发重渲染
  expandedLockCreds.value = {
    ...expandedLockCreds.value,
    [hostname]: !expandedLockCreds.value[hostname],
  };
};
/** 凭据展示文本。掩码时保留开头几位（secret 的 cfast_ 前缀、client id 前 4 位），
 *  这样一眼能分清哪个是账号、哪个是密码；点击复制拿到的始终是完整值，不受掩码影响。 */
const credDisplay = (value: string, hostname: string): string => {
  if (!value) return '';
  if (isLockCredExpanded(hostname)) return value;
  return value.slice(0, value.startsWith('cfast_') ? 6 : 4) + '••••••••••••';
};

// 上锁 / 换密码成功后的凭据展示弹窗
const showLockInfoModal = ref(false);
const lockInfoDraft = ref<{ hostname: string; clientId: string; clientSecret: string } | null>(null);

// 配置页的 Access Token：留空 = 用「授权登录」凭证。
// 该凭证需要 Access: Apps and Policies 与 Access: Service Tokens 两个编辑权限（见 README）。
const accessTokenInput = ref(localStorage.getItem('access_api_token') || '');
const saveAccessToken = () => {
  accessTokenInput.value = accessTokenInput.value.trim();
  localStorage.setItem('access_api_token', accessTokenInput.value);
  appendLog(`[INFO] ${t.value.logs.access_token_saved_log}`, 'info', 'misc');
  showToast(t.value.misc_tab.access_token_saved);
};

// 上锁（底层）：调后端创建 Access 应用 + Service Token，成功后按域名落库并展示凭据
const doLock = async (hostname: string): Promise<boolean> => {
  try {
    const res = await invoke<{
      hostname: string; app_uid: string; app_name: string;
      token_uid: string; policy_uid: string; client_id: string; client_secret: string;
    }>('tunnel_lock', { hostname, accessToken: accessTokenInput.value || null });
    tunnelLocks.value[res.hostname] = {
      hostname: res.hostname,
      appUid: res.app_uid,
      appName: res.app_name,
      tokenUid: res.token_uid,
      policyUid: res.policy_uid,
      clientId: res.client_id,
      clientSecret: res.client_secret,
    };
    persistTunnelLocks();
    lockInfoDraft.value = { hostname: res.hostname, clientId: res.client_id, clientSecret: res.client_secret };
    showLockInfoModal.value = true;
    appendLog(`[SUCCESS] ${fmt(t.value.logs.lock_ok, { host: res.hostname })}`, 'success', 'server');
    return true;
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.lock_failed, { err: errorText(err) })}`, 'error', 'server');
    showToast(`${errorText(err)}`);
    return false;
  }
};

// 解锁（底层）：删掉云端的 Access 应用与 Service Token，恢复公开访问。
// quiet：批量清理（解绑域名 / 删除隧道）时不逐个弹失败 toast，由调用方汇总提示。
const doUnlock = async (entry: TunnelLockEntry, opts: { quiet?: boolean } = {}): Promise<boolean> => {
  try {
    const res = await invoke<string>('tunnel_unlock', {
      appUid: entry.appUid,
      tokenUid: entry.tokenUid,
      policyUid: entry.policyUid || null,
      accessToken: accessTokenInput.value || null,
    });
    delete tunnelLocks.value[entry.hostname];
    persistTunnelLocks();
    appendLog(`[SUCCESS] ${res} (${entry.hostname})`, 'success', 'server');
    return true;
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.unlock_failed, { err: errorText(err) })}`, 'error', 'server');
    if (!opts.quiet) showToast(`${errorText(err)}`);
    return false;
  }
};

// 换密码（底层）：删旧锁 + 重新上锁，旧密码立即作废
const doRotatePassword = async (entry: TunnelLockEntry): Promise<boolean> => {
  try {
    const res = await invoke<{
      hostname: string; app_uid: string; app_name: string;
      token_uid: string; policy_uid: string; client_id: string; client_secret: string;
    }>('tunnel_rotate_password', {
      hostname: entry.hostname,
      appUid: entry.appUid,
      tokenUid: entry.tokenUid,
      policyUid: entry.policyUid || null,
      accessToken: accessTokenInput.value || null,
    });
    tunnelLocks.value[res.hostname] = {
      hostname: res.hostname,
      appUid: res.app_uid,
      appName: res.app_name,
      tokenUid: res.token_uid,
      policyUid: res.policy_uid,
      clientId: res.client_id,
      clientSecret: res.client_secret,
    };
    persistTunnelLocks();
    lockInfoDraft.value = { hostname: res.hostname, clientId: res.client_id, clientSecret: res.client_secret };
    showLockInfoModal.value = true;
    appendLog(`[SUCCESS] ${fmt(t.value.logs.rotate_ok, { host: res.hostname })}`, 'success', 'server');
    return true;
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.rotate_failed, { err: errorText(err) })}`, 'error', 'server');
    showToast(`${errorText(err)}`);
    return false;
  }
};

// 行内「上锁」：给指定的单个域名上锁
const promptLockHostname = async (hostname: string) => {
  if (isLockMutating.value || lockOf(hostname)) return;
  soundManager.playSuccess();
  isLockMutating.value = true;
  try {
    await doLock(hostname);
  } finally {
    isLockMutating.value = false;
  }
};

// 行内「解锁」：弹确认框（解锁后域名恢复公开，任何知道域名的人都能连）
const unlockTarget = ref<string>(''); // 存 hostname
const showUnlockModal = ref(false);

const promptUnlockHostname = (hostname: string) => {
  if (isLockMutating.value) return;
  soundManager.playClick();
  unlockTarget.value = hostname;
  showUnlockModal.value = true;
};

const confirmUnlockHostname = async () => {
  const hostname = unlockTarget.value;
  const entry = hostname ? lockOf(hostname) : null;
  if (!hostname || !entry || isLockMutating.value) {
    showUnlockModal.value = false;
    return;
  }
  showUnlockModal.value = false;
  isLockMutating.value = true;
  try {
    await doUnlock(entry);
    showToast(t.value.server_tab.unlocked_toast);
  } finally {
    isLockMutating.value = false;
  }
};

// 行内「换密码」：弹确认框（旧密码会作废，正在连的客户端会断）
const rotateTarget = ref<string>(''); // 存 hostname
const showRotateModal = ref(false);

const promptRotatePassword = (hostname: string) => {
  if (isLockMutating.value) return;
  soundManager.playClick();
  rotateTarget.value = hostname;
  showRotateModal.value = true;
};

const confirmRotatePassword = async () => {
  const hostname = rotateTarget.value;
  const entry = hostname ? lockOf(hostname) : null;
  if (!hostname || !entry || isLockMutating.value) {
    showRotateModal.value = false;
    return;
  }
  showRotateModal.value = false;
  isLockMutating.value = true;
  try {
    await doRotatePassword(entry);
  } finally {
    isLockMutating.value = false;
  }
};

// 「已绑定域名」管理列表：只展开固定域名的绑定记录，
// 云端托管隧道不在此处管理（其 ingress 由 Cloudflare 侧维护）。
// 按隧道聚合：一个隧道一行，其下所有绑定的域名收在 records 里，
// 避免同一个隧道有 N 个域名就重复出现 N 行隧道名（原表格太乱）。
type DnsBoundGroup = {
  tunnelId: string;
  tunnelName: string;
  records: DnsBoundRow[];
};

const dnsBoundGroups = computed<DnsBoundGroup[]>(() =>
  serverTunnelList.value
    .map(t => ({
      tunnelId: t.id,
      tunnelName: t.name,
      records: (t.hostnames || []).map(h => ({
        recordId: h.id,
        hostname: h.name,
        tunnelName: t.name,
        tunnelId: t.id,
      })),
    }))
    .filter(g => g.records.length > 0),
);

// ================ 域名清理：解绑单个域名 / 删除整条隧道共用 ================
//
// 密码锁挂在「域名」上（Access 应用按域名建），所以域名一旦被解绑、或它所属的隧道
// 被删除，这把锁就必须跟着消失 —— 否则云端会残留一个仍在拦截该域名的 Access 应用，
// 而软件里再也定位不到它（域名记录没了，锁记录也成了孤儿）。
// 下面三个函数把这段逻辑收在一处，两条删除路径走同一套代码。

/** 一条绑定域名：清锁只需要 hostname，兜底补删 DNS 记录需要 recordId */
type BoundDomain = { recordId: string; hostname: string };

/** 取某条隧道当前绑定的域名（按域名去重）。
 *  **必须在删隧道之前调用** —— 隧道一删，列表里就查不到它绑了哪些域名了。 */
const collectBoundDomains = (tunnelId: string): BoundDomain[] => {
  const out = new Map<string, BoundDomain>();
  for (const h of tunnelList.value.find(x => x.id === tunnelId)?.hostnames || []) {
    out.set(h.name, { recordId: h.id, hostname: h.name });
  }
  // 隧道列表与 DNS 绑定面板是两份视图，取并集，避免其中一处没刷新时漏掉域名
  for (const g of dnsBoundGroups.value) {
    if (g.tunnelId !== tunnelId) continue;
    for (const r of g.records) {
      if (!out.has(r.hostname)) out.set(r.hostname, { recordId: r.recordId, hostname: r.hostname });
    }
  }
  return [...out.values()];
};

/** 核对这些域名里还有哪些 DNS 记录残留在云端：删隧道时后端会顺手清理绑定的域名，
 *  但拿不到 tunnel id（未授权登录等）时会跳过，这里按快照兜底。查询失败就交给调用方硬删。 */
const findLeftoverDomains = async (domains: BoundDomain[]): Promise<BoundDomain[]> => {
  if (domains.length === 0) return [];
  try {
    const map = await invoke<Record<string, DnsBinding[]>>('get_tunnel_hostnames');
    const alive = new Set<string>();
    for (const list of Object.values(map)) for (const b of list) alive.add(b.id);
    return domains.filter(d => alive.has(d.recordId));
  } catch (err: any) {
    appendLog(`[WARN] ${fmt(t.value.logs.leftover_check_failed, { err: errorText(err) })}`, 'warn', 'server');
    return domains;
  }
};

/** 清掉这些域名上的密码锁：云端的 Access 应用 + 策略 + Service Token，成功后连本地记录一起删。
 *  云端删失败时**保留**本地记录 —— 里面存着资源 ID，该域名重新绑定后还能再点解锁重试。 */
const purgeDomainLocks = async (
  domains: BoundDomain[],
): Promise<{ done: string[]; failed: string[] }> => {
  const done: string[] = [];
  const failed: string[] = [];
  if (domains.length === 0) return { done, failed };
  const locked = domains.filter(d => lockOf(d.hostname));
  if (locked.length === 0) return { done, failed };
  isLockMutating.value = true;
  try {
    for (const d of locked) {
      const lock = lockOf(d.hostname);
      if (!lock) continue;
      // quiet：批量清理不逐个弹 toast，成败由调用方汇总成一行日志 + 一条提示
      const ok = await doUnlock(lock, { quiet: true });
      (ok ? done : failed).push(d.hostname);
    }
  } finally {
    isLockMutating.value = false;
  }
  return { done, failed };
};

// 控制台高度与拖拽调整逻辑
const consoleHeight = ref(Number(localStorage.getItem('console_height')) || 170);
let isResizing = false;
let startY = 0;
let startHeight = 170;

const startConsoleResize = (e: MouseEvent) => {
  isResizing = true;
  startY = e.clientY;
  startHeight = consoleHeight.value;
  document.addEventListener('mousemove', onConsoleResize);
  document.addEventListener('mouseup', stopConsoleResize);
  document.body.style.userSelect = 'none';
};

const onConsoleResize = (e: MouseEvent) => {
  if (!isResizing) return;
  const deltaY = startY - e.clientY;
  // 限制控制台高度在 80px 至 480px 之间
  const newHeight = Math.min(Math.max(startHeight + deltaY, 80), 480);
  consoleHeight.value = newHeight;
};

const stopConsoleResize = () => {
  if (!isResizing) return;
  isResizing = false;
  document.removeEventListener('mousemove', onConsoleResize);
  document.removeEventListener('mouseup', stopConsoleResize);
  document.body.style.userSelect = '';
  localStorage.setItem('console_height', consoleHeight.value.toString());
};

// 弹窗与 Toast
const showDeleteModal = ref(false);

// 待删除的隧道（记录 id 是给「连带删除提示」数域名用的）
const pendingDelete = ref<{ id: string; name: string } | null>(null);

// 确认弹窗正文。原先分「本地 / 云端」两套文案，视图合并后只剩一套；
// {name} / {target} 两种占位符都替换一遍，免得换文案时漏改一个。
const deleteConfirmMessage = computed(() => {
  const name = pendingDelete.value?.name || '';
  return String(t.value.server_tab.errors.delete_confirm_msg)
    .replace('{name}', name)
    .replace('{target}', name);
});

// 删除确认框里的「连带删除」提示：这条隧道绑了几个域名、其中几把密码锁会被一起删掉。
// 域名一条都没有、也没锁时不显示这行。
const deleteCascadeHint = computed(() => {
  const id = pendingDelete.value?.id;
  if (!id) return '';
  const domains = collectBoundDomains(id);
  const locks = domains.filter(d => lockOf(d.hostname)).length;
  if (domains.length === 0) return '';
  return t.value.server_tab.delete_cascade_hint
    .replace('{domains}', String(domains.length))
    .replace('{locks}', String(locks));
});

const showExitConfirmModal = ref(false);
const toastMessage = ref('');
const consoleBodyRef = ref<HTMLDivElement | null>(null);

// 日志记录
const logs = ref<LogEntry[]>([
  {
    id: 'init-1',
    timestamp: new Date().toLocaleTimeString(),
    message: t.value.logs.app_ready,
    level: 'info',
    source: 'system',
  },
]);

// 格式校验触发
const onClientFormDomainInput = () => {
  const val = clientForm.value.domain;
  clientFormDomainHasError.value = val.length > 0 && !isDomainValid(val);
};

const onClientFormPortInput = () => {
  const val = clientForm.value.port;
  clientFormPortHasError.value = val.length > 0 && !isPortValid(val);
  localStorage.setItem('client_port', val);
};

const onDnsRouteDomainInput = () => {
  const val = dnsRoute.value.domain;
  dnsRouteDomainHasError.value = val.length > 0 && !isDomainValid(val);
  localStorage.setItem('dns_route_domain', val);
};

// 切换主题 (带 360° 旋转动效)
const toggleTheme = () => {
  isThemeSpinning.value = true;
  isDarkMode.value = !isDarkMode.value;
  localStorage.setItem('app_theme', isDarkMode.value ? 'dark' : 'light');
  syncThemeToDocument();
  setTimeout(() => {
    isThemeSpinning.value = false;
  }, 500);
};

// 切换 Tab
const switchTab = (tab: string) => {
  currentTab.value = tab;
};

// 显示 Toast
const showToast = (msg: string) => {
  toastMessage.value = msg;
  setTimeout(() => {
    if (toastMessage.value === msg) {
      toastMessage.value = '';
    }
  }, 2600);
};

// 控制台每行左侧已经按 level 渲染了一个 [INFO]/[WARN]/[ERROR]/[SUCCESS] 标签，
// 而各处调用点（含 Rust 侧 emit 过来的）又在正文开头写了一遍同样的前缀，
// 于是同一行会显示成「[WARN][INFO] 临时链接 已停止…」这种两层甚至多层标签。
// 统一在入库前剥掉正文开头的层级标签（只剥开头，正文中间出现的 [81044] 之类原样保留），
// 并把 cloudflared 原样吐出的多行 stderr 压成一行 —— 保证「一行日志 = 一个标签 + 一句正文」。
const LOG_LEVEL_PREFIX = /^(?:\s*\[(?:INFO|WARN|ERROR|SUCCESS|DEBUG)\])+/i;
const normalizeLogMessage = (raw: string) =>
  raw.replace(LOG_LEVEL_PREFIX, '').replace(/\s+/g, ' ').trim();

// 控制台最多保留的日志行数（滑动窗口，只留最近的）。
// 日志面板是本软件唯一会「随时间单向增长」的内存：每条日志既是数组里的一个对象，
// 又是 DOM 里的一行，而原先只 push、从不回收 —— 挂一整天能堆到几千条，
// 每条还都带着 Vue 的响应式代理。超出上限就从头部丢，复制日志也只复制保留下来的部分。
const MAX_LOG_ENTRIES = 500;

// 追加日志
const appendLog = (message: string, level: LogEntry['level'] = 'info', source: LogEntry['source'] = 'system') => {
  logs.value.push({
    id: `${Date.now()}-${Math.random().toString(36).substr(2, 6)}`,
    timestamp: new Date().toLocaleTimeString(),
    message: normalizeLogMessage(message),
    level,
    source,
  });

  if (logs.value.length > MAX_LOG_ENTRIES) {
    logs.value.splice(0, logs.value.length - MAX_LOG_ENTRIES);
  }

  nextTick(() => {
    if (consoleBodyRef.value) {
      consoleBodyRef.value.scrollTop = consoleBodyRef.value.scrollHeight;
    }
  });
};

const clearLogs = () => {
  logs.value = [];
  showToast(t.value.server_tab.btn_clear_log);
};

const copyLogs = async () => {
  const fullText = logs.value.map(l => `[${l.timestamp}] [${l.level.toUpperCase()}] ${l.message}`).join('\n');
  try {
    await navigator.clipboard.writeText(fullText);
    showToast(t.value.console.copy_logs_success);
  } catch {
    appendLog(t.value.logs.copy_logs_failed, 'error');
  }
};

// 打开外部链接
const openUrl = async (url: string) => {
  try {
    await invoke('open_external_url', { url });
    appendLog(`[INFO] ${fmt(t.value.logs.opened_in_browser, { url })}`, 'info', 'misc');
  } catch (err) {
    // err 是 unknown（TS 4.4+ 的 catch 默认类型），不能直接塞进 fmt —— 走 errorText 转字符串
    appendLog(`${fmt(t.value.logs.open_link_failed, { err: errorText(err) })}`, 'error', 'misc');
  }
};

// 打开 cloudflared 的默认凭证目录（%USERPROFILE%\.cloudflared）
const handleOpenConfigDir = async () => {
  try {
    const dir = await invoke<string>('open_cloudflared_config_dir');
    showToast(`${fmt(t.value.logs.config_dir_opened, { dir })}`);
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.config_dir_open_failed, { err })}`, 'error', 'misc');
  }
};

// 选择隧道
const selectTunnel = (tunnel: TunnelInfo) => {
  selectedTunnel.value = tunnel;
};

// 把绑定域名补进当前列表。
// 这一步要打 Cloudflare API（遍历 zone 下的 DNS 记录），慢且可能失败，
// 所以与列表本体解耦：失败时**保留上一次的值** —— 网络抖一下就把整列刷成
// 「未绑定」，会让人误以为域名被解绑了，比不更新更糟。
const fillTunnelHostnames = async () => {
  try {
    const map = await invoke<Record<string, DnsBinding[]>>('get_tunnel_hostnames');
    // 重建数组而不是原地改属性，保证表格一定收到更新
    tunnelList.value = tunnelList.value.map(t => ({ ...t, hostnames: map[t.id] ?? [] }));
  } catch {
    // 未授权 / 网络不通：静默忽略，保留旧值
  }
};

// 刷新隧道列表
// scope：由哪个列表的刷新按钮触发（local=固定域名列表 / remote=云端托管列表），
//        只影响日志文案、统计口径，以及要不要顺带刷新云端数据；内部调用不传则统计全部。
//
// 这里最要紧的是「别让网络拖着按钮」：列表本体读的是本地 cloudflared 配置（毫秒级），
// 而绑定域名、云端 ingress 都要打 Cloudflare API —— 网络不通时单次能一直等到超时。
// 所以列表先上屏、慢活全部异步化，按钮随后就能恢复可点。
const handleRefreshTunnels = async () => {
  // 防重入：连点不做第二次
  if (refreshingTunnels.value.local) return;
  refreshingTunnels.value.local = true;
  try {
    const res = await invoke<TunnelInfo[]>('list_tunnels');
    tunnelList.value = res;

    // 绑定域名走云 API，不阻塞列表上屏与按钮恢复
    void fillTunnelHostnames();

    // 与后端对账固定隧道的运行状态（多开后靠这里把已退出的进程同步掉）
    await reconcileServerRunning();

    appendLog(`[INFO] ${fmt(t.value.logs.tunnel_list_refreshed, { count: res.length })}`, 'info', 'server');
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.tunnel_list_refresh_failed, { err })}`, 'error', 'server');
  } finally {
    refreshingTunnels.value.local = false;
  }
};

// 源站目标描述文案（日志 / 提示用）
const describeServerTarget = (protocol: string, port: string, unixSocket: string) => {
  if (protocol === 'hello_world') return t.value.logs.target_hello_world;
  if (protocol === 'unix' || protocol === 'unix+tls') return `${protocol}:${unixSocket}`;
  return `${protocol}://127.0.0.1:${port}`;
};

// 隧道对象本身不含源站协议/端口（Cloudflare 侧只存隧道，ingress 在本地 config.yml），
// 所以按隧道名记住「上次启动用的配置」，供列表行内启动按钮直接启动该条隧道
const tunnelCfgKey = (name: string) => `server_cfg_${name}`;

const loadTunnelCfg = (name: string): { protocol: string; port: string; unixSocket: string } | null => {
  try {
    const raw = localStorage.getItem(tunnelCfgKey(name));
    if (!raw) return null;
    const obj = JSON.parse(raw);
    if (!obj || typeof obj.protocol !== 'string') return null;
    return {
      protocol: obj.protocol,
      port: String(obj.port ?? ''),
      unixSocket: String(obj.unixSocket ?? ''),
    };
  } catch {
    return null;
  }
};

// 显式保存某条隧道的源站配置（创建 / 修改弹窗共用）。
// 早先版本从顶部常驻表单取值，表单移除后由调用方把四个值直接传进来。
const saveTunnelCfgValues = (name: string, protocol: string, port: string, unixSocket: string) => {
  try {
    localStorage.setItem(
      tunnelCfgKey(name),
      JSON.stringify({ protocol, port, unixSocket }),
    );
  } catch {}
};

// 列表行内「启动」：用该隧道保存的源站配置直接启动。
// 没有保存过配置（旧版本创建的隧道）时引导用户先点「修改」补全。
const handleRowStart = async (tunnel: TunnelInfo) => {
  const name = tunnel.name.trim();
  const saved = loadTunnelCfg(name);
  if (!saved) {
    soundManager.playClick();
    appendLog(`[WARN] ${fmt(t.value.logs.tunnel_need_config, { name })}`, 'warn', 'server');
    showToast(t.value.server_tab.edit_need_config);
    openTunnelEditModal(tunnel);
    return;
  }
  if (saved.protocol === 'unix' || saved.protocol === 'unix+tls') {
    if (!saved.unixSocket) {
      appendLog(`[ERROR] ${t.value.logs.unix_socket_required}`, 'error', 'server');
      return;
    }
  } else if (saved.protocol !== 'hello_world' && !isPortValid(saved.port)) {
    appendLog(`[ERROR] ${fmt(t.value.logs.tunnel_port_invalid, { name })}`, 'error', 'server');
    openTunnelEditModal(tunnel);
    return;
  }

  soundManager.playSuccess();
  try {
    await invoke<string>('start_server_tunnel', {
      name,
      port: saved.port,
      protocol: saved.protocol,
      unixSocket: saved.unixSocket,
    });
    markServerRunning(name);
    showToast(`${fmt(t.value.logs.tunnel_started, { name, target: describeServerTarget(saved.protocol, saved.port, saved.unixSocket) })}`);
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.tunnel_start_failed, { err })}`, 'error', 'server');
  }
};

// 停止服务端隧道 (普通点击音效)
// 省略 name 时停「表单里当前这条」；列表行内按钮会传入该行隧道名，多开时逐条停
const handleStopServer = async (name?: string) => {
  const target = (typeof name === 'string' ? name : serverConfig.value.name).trim();
  if (!target) {
    appendLog(`[ERROR] ${t.value.logs.no_tunnel_to_stop}`, 'error', 'server');
    return;
  }
  soundManager.playClick();
  try {
    // 停止成功的日志由 Rust 侧统一广播，这里不再重复打印
    await invoke<string>('stop_server_tunnel', { name: target });
    markServerStopped(target);
    showToast(`${fmt(t.value.logs.tunnel_stopped, { name: target })}`);
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.tunnel_stop_failed, { name: target, err })}`, 'error', 'server');
  }
};

// 打开「添加绑定」弹窗（隧道下拉 + 域名输入）
const openDnsAddModal = () => {
  // 没有任何固定域名隧道时无处可绑：直接提示，不弹空下拉框
  if (serverTunnelList.value.length === 0) {
    soundManager.playClick();
    appendLog(`[ERROR] ${t.value.logs.no_tunnel_for_dns}`, 'error', 'server');
    showToast(t.value.server_tab.quick_list_empty);
    return;
  }
  soundManager.playClick();
  // 预选第一个固定域名隧道（若有），域名清空
  dnsRoute.value.name = serverTunnelList.value[0]?.name || '';
  dnsRoute.value.domain = '';
  dnsRouteNameHasError.value = false;
  dnsRouteDomainHasError.value = false;
  showDnsAddModal.value = true;
};

const cancelDnsAdd = () => {
  showDnsAddModal.value = false;
  dnsRouteDomainHasError.value = false;
};

// 提交添加绑定（复用 handleRouteDns 的校验与调用，成功才关闭弹窗）
const confirmDnsAdd = async () => {
  if (isDnsMutating.value) return;
  const ok = await handleRouteDns();
  if (ok) {
    showDnsAddModal.value = false;
  }
};

// 绑定 DNS 路由 (cloudflared tunnel route dns <name> <hostname>)
// 返回是否绑定成功（供添加弹窗据此决定是否关闭）
const handleRouteDns = async (): Promise<boolean> => {
  const name = dnsRoute.value.name.trim();
  const domain = dnsRoute.value.domain.trim();

  if (!isTunnelNameValid(name)) {
    dnsRouteNameHasError.value = true;
    appendLog(`[ERROR] ${t.value.server_tab.errors.tunnel_invalid}`, 'error', 'server');
    return false;
  }
  if (!isDomainValid(domain)) {
    dnsRouteDomainHasError.value = true;
    appendLog(`[ERROR] ${t.value.server_tab.errors.dns_domain_invalid}`, 'error', 'server');
    return false;
  }

  soundManager.playSuccess();
  try {
    const res = await invoke<string>('route_dns_tunnel', { name, hostname: domain });
    appendLog(`[SUCCESS] ${res}`, 'success', 'server');
    showToast(`${fmt(t.value.logs.dns_bound_ok, { domain, name })}`);
    // 绑定后立即刷新域名列表，新域名马上出现在「已绑定域名」里
    await refreshHostnamesOnly();
    return true;
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.dns_bind_failed, { err })}`, 'error', 'server');
    showToast(`${fmt(t.value.logs.bind_failed_toast, { err })}`);
    return false;
  }
};

// 只重新拉取绑定域名并回填到已有隧道列表（不重拉隧道列表、不写刷新日志），
// 用于绑定 / 改名 / 解绑后同步界面显示。
const refreshHostnamesOnly = async () => {
  try {
    const map = await invoke<Record<string, DnsBinding[]>>('get_tunnel_hostnames');
    for (const t of tunnelList.value) t.hostnames = map[t.id] || [];
  } catch (err: any) {
    appendLog(`[WARN] ${fmt(t.value.logs.refresh_hostnames_failed, { err })}`, 'warn', 'server');
  }
};

// 打开「修改绑定域名」弹窗（预填当前域名）
const promptEditDnsRoute = (row: DnsBoundRow) => {
  soundManager.playClick();
  dnsEditTarget.value = row;
  dnsEditValue.value = row.hostname;
  dnsEditHasError.value = false;
  showDnsEditModal.value = true;
};

const cancelEditDnsRoute = () => {
  showDnsEditModal.value = false;
  dnsEditTarget.value = null;
  dnsEditValue.value = '';
  dnsEditHasError.value = false;
};

// 提交改名：只 PATCH DNS 记录的 name 字段，不改动隧道 ingress 配置。
// 密码锁挂在域名上，改了名就等于换了一个域名：旧域名的锁必须删掉（Access 应用是
// 按域名建的，留着只会变成云端孤儿）；新域名要保护就重新点一次「上锁」。
const confirmEditDnsRoute = async () => {
  const target = dnsEditTarget.value;
  if (!target || isDnsMutating.value) return;

  const next = dnsEditValue.value.trim();
  if (!isDomainValid(next)) {
    dnsEditHasError.value = true;
    appendLog(`[ERROR] ${t.value.server_tab.errors.dns_domain_invalid}`, 'error', 'server');
    return;
  }
  // 没改动就直接关掉，不发无意义的请求
  if (next === target.hostname) {
    cancelEditDnsRoute();
    return;
  }

  isDnsMutating.value = true;
  try {
    const res = await invoke<string>('rename_dns_route', {
      recordId: target.recordId,
      hostname: next,
    });
    appendLog(`[SUCCESS] ${res}`, 'success', 'server');

    // 改名成功后才清旧锁：万一改名失败，锁还挂在那儿，不会出现「名字没换成、保护先没了」
    const { done, failed } = await purgeDomainLocks([
      { recordId: target.recordId, hostname: target.hostname },
    ]);
    if (failed.length) {
      appendLog(
        `[WARN] ${fmt(t.value.logs.rename_lock_purge_failed, { host: target.hostname })}`,
        'warn',
        'server',
      );
    }

    showToast(
      `${fmt(t.value.logs.rename_ok, { from: target.hostname, to: next })}${done.length ? t.value.logs.rename_ok_lock_purged : ''}`,
    );
    cancelEditDnsRoute();
    await refreshHostnamesOnly();
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.rename_domain_failed, { err })}`, 'error', 'server');
    showToast(`${fmt(t.value.logs.rename_domain_failed_toast, { err })}`);
  } finally {
    isDnsMutating.value = false;
  }
};

// 打开「解除绑定」确认弹窗
const promptUnbindDnsRoute = (row: DnsBoundRow) => {
  soundManager.playClick();
  dnsUnbindTarget.value = row;
  showDnsUnbindModal.value = true;
};

const cancelUnbindDnsRoute = () => {
  showDnsUnbindModal.value = false;
  dnsUnbindTarget.value = null;
};

/** 解绑域名时，把云端 ingress（「已发布应用程序路由」）里指向该域名的规则摘掉。
 *  一个域名在两处都留了痕迹：DNS 的 CNAME 记录 + 隧道的 ingress 规则。只删前者的话，
 *  域名没了、ingress 里那条 hostname → 本机服务的规则会一直挂在「已发布应用程序路由」里，
 *  只能进「修改隧道」手动删。
 *  返回是否真的删掉了（云端本来就没有这条时返回 false，也就不必写回）。 */
const stripIngressHostname = async (tunnelId: string, hostname: string): Promise<boolean> => {
  const cfg = await invoke<{ rules: TunnelIngressRule[] }>('fetch_tunnel_config', { tunnelId });
  const rules = cfg?.rules ?? [];
  const want = hostname.trim().toLowerCase();
  // 其余规则原样带回（含 path 与反解不出的 raw service），只摘掉域名匹配的那几条
  const kept = rules.filter(r => (r.hostname || '').trim().toLowerCase() !== want);
  if (kept.length === rules.length) return false;
  // 摘掉之后一条不剩时补一条兜底：Cloudflare 不接受空的 ingress，
  // 真发个空数组上去会整条写回失败 —— 那就成了「DNS 删了、ingress 还留着」，正是要修的状态
  if (kept.length === 0) kept.push({ service: 'http_status:404' } as TunnelIngressRule);
  await invoke<string>('update_tunnel_config', { tunnelId, ingress: kept });
  return true;
};

// 解绑：① 删掉该域名的密码锁（锁挂在域名上，DNS 记录一删就再也定位不到它的 Access 应用）
//      ② 删 Cloudflare 侧的 CNAME 记录
//      ③ 摘掉隧道云端 ingress 里指向该域名的规则，让它同时从「已发布应用程序路由」里消失
// 隧道本身不动。
const confirmUnbindDnsRoute = async () => {
  const target = dnsUnbindTarget.value;
  if (!target || isDnsMutating.value) return;

  isDnsMutating.value = true;
  try {
    const { done, failed } = await purgeDomainLocks([
      { recordId: target.recordId, hostname: target.hostname },
    ]);
    if (failed.length) {
      appendLog(
        `[WARN] ${fmt(t.value.logs.unbind_lock_purge_failed, { host: target.hostname })}`,
        'warn',
        'server',
      );
    }

    const res = await invoke<string>('delete_dns_route', { recordId: target.recordId });
    appendLog(`[SUCCESS] ${res} (${target.hostname})`, 'success', 'server');

    // 放在删 DNS 之后：万一前面失败，不会留下「路由没了、DNS 还在」的半截状态
    // —— 那会让域名照常解析到隧道，却因为没有 ingress 规则而回 404，更难查。
    let ingressRemoved = false;
    try {
      ingressRemoved = await stripIngressHostname(target.tunnelId, target.hostname);
      if (ingressRemoved) {
        appendLog(
          `[SUCCESS] ${fmt(t.value.logs.unbind_ingress_removed, { name: target.tunnelName, host: target.hostname })}`,
          'success',
          'server',
        );
      }
    } catch (err: any) {
      appendLog(
        `[WARN] ${fmt(t.value.logs.unbind_ingress_remove_failed, { host: target.hostname, name: target.tunnelName, err: errorText(err) })}`,
        'warn',
        'server',
      );
    }

    showToast(
      `${fmt(t.value.logs.unbind_ok, { host: target.hostname })}${done.length ? t.value.logs.unbind_ok_lock_purged : ''}${ingressRemoved ? t.value.logs.unbind_ok_ingress_removed : ''}`,
    );
    cancelUnbindDnsRoute();
    await refreshHostnamesOnly();
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.unbind_domain_failed, { err })}`, 'error', 'server');
    showToast(`${fmt(t.value.logs.unbind_domain_failed_toast, { err })}`);
  } finally {
    isDnsMutating.value = false;
  }
};

// 启动临时链接（临时域名）。返回是否启动成功（创建弹窗据此决定是否关闭）。
const handleStartQuick = async (): Promise<boolean> => {
  const port = quickConfig.value.port.trim();
  const protocol = quickConfig.value.protocol;
  const unixSocket = quickConfig.value.unixSocket.trim();

  if (protocol === 'unix' || protocol === 'unix+tls') {
    if (!unixSocket) {
      appendLog(`[ERROR] ${t.value.logs.unix_socket_required}`, 'error', 'quick');
      return false;
    }
  } else if (protocol !== 'hello_world' && !isPortValid(port)) {
    quickPortHasError.value = true;
    appendLog(`[ERROR] ${t.value.logs.quick_port_invalid}`, 'error', 'quick');
    return false;
  }
  soundManager.playSuccess();
  // key 需与后端进程表一致：hello_world 固定 key；unix 用 协议:套接字路径
  const key = protocol === 'hello_world'
    ? 'hello_world'
    : (protocol === 'unix' || protocol === 'unix+tls')
      ? `${protocol}:${unixSocket}`
      : `${protocol}://127.0.0.1:${port}`;
  const displayPort = (protocol === 'unix' || protocol === 'unix+tls') ? unixSocket : port;
  try {
    // 启动成功的日志由 Rust 侧统一广播，这里不再重复打印
    await invoke<string>('start_quick_tunnel', { port, protocol, unixSocket });
    localStorage.setItem('quick_port', port);
    localStorage.setItem('quick_protocol', protocol);
    localStorage.setItem('quick_unix_socket', unixSocket);
    // 移除同 key 的旧条目，新增一条「启动中」状态的条目
    quickTunnels.value = quickTunnels.value.filter(t => t.key !== key);
    quickTunnels.value.push({ key, protocol, port: displayPort, url: '', status: 'starting' });
    showToast(t.value.logs.quick_started);
    return true;
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.quick_start_failed, { err })}`, 'error', 'quick');
    return false;
  }
};

// 临时链接创建弹窗：协议 + 端口/套接字，确认即启动（临时域名由 Cloudflare 随机分配）
const showQuickCreateModal = ref(false);

const openQuickCreateModal = () => {
  soundManager.playClick();
  quickPortHasError.value = false;
  showQuickCreateModal.value = true;
};

const confirmQuickCreate = async () => {
  const ok = await handleStartQuick();
  if (ok) showQuickCreateModal.value = false;
};

// 刷新临时链接列表（从后端同步运行状态）
const refreshQuickTunnels = async () => {
  // 防重入：上一轮刷新还没结束就不再叠加（临时链接走的是本地命令 is_quick_running，毫秒级，
  // 但连点仍会并发触发，这里挡住）
  if (refreshingTunnels.value.quick) return;
  refreshingTunnels.value.quick = true;
  try {
    const quickKeys = await invoke<string[]>('is_quick_running');
    // 保留已有 url 信息，合并后端返回的运行中 key
    const merged: QuickTunnelItem[] = quickKeys.map(key => {
      const { protocol, port } = parseQuickKey(key);
      const existing = quickTunnels.value.find(t => t.key === key);
      return {
        key,
        protocol,
        port,
        url: existing?.url || '',
        status: 'running' as const,
      };
    });
    quickTunnels.value = merged;
    appendLog(`[INFO] ${fmt(t.value.logs.quick_list_refreshed, { count: merged.length })}`, 'info', 'quick');
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.quick_list_refresh_failed, { err })}`, 'error', 'quick');
  } finally {
    refreshingTunnels.value.quick = false;
  }
};

// 点击「停止」仅弹二次确认框，不直接停止
const promptStopQuick = (key: string) => {
  soundManager.playClick();
  quickStopKey.value = key;
  showQuickStopModal.value = true;
};

const cancelStopQuick = () => {
  showQuickStopModal.value = false;
  quickStopKey.value = '';
};

// 确认停止指定临时链接（按 key）
const confirmStopQuick = async () => {
  const key = quickStopKey.value;
  if (!key) return;
  showQuickStopModal.value = false;
  quickStopKey.value = '';
  try {
    // 停止成功的日志由 Rust 侧统一广播（含「临时域名已失效」），这里不再重复打印
    await invoke<string>('stop_quick_tunnel', { key });
    quickTunnels.value = quickTunnels.value.filter(t => t.key !== key);
    showToast(t.value.logs.quick_stopped);
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.quick_stop_failed, { err })}`, 'error', 'quick');
  }
};

// 复制临时链接临时链接（点击临时域名触发）
const copyQuickUrl = async (url: string) => {
  try {
    await navigator.clipboard.writeText(url);
    showToast(t.value.logs.quick_url_copied);
  } catch {
    appendLog(t.value.logs.quick_url_copy_failed, 'error', 'quick');
  }
};

// 复制绑定域名（点击列表里的域名标签触发）
const copyHostname = async (hostname: string) => {
  try {
    await navigator.clipboard.writeText(hostname);
    showToast(`${fmt(t.value.logs.hostname_copied, { hostname })}`);
  } catch {
    appendLog(`${fmt(t.value.logs.hostname_copy_failed, { hostname })}`, 'error', 'server');
  }
};

// 通用复制（访问账号 / 访问密码等），成功与否都给一行提示
const copyText = async (text: string) => {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    showToast(t.value.server_tab.copied_toast);
  } catch {
    appendLog(t.value.logs.copy_failed_clipboard, 'error', 'server');
  }
};

// 删除隧道确认流程（固定域名隧道 / 云端托管隧道共用）。
// 固定域名列表的删除按钮在每一行里，直接把该行隧道传进来；
// 不带参数时（若有调用方）回退到当前选中的行。
const promptDeleteTunnel = (tunnel?: TunnelInfo) => {
  const target = tunnel ?? selectedTunnel.value;
  if (!target) {
    showToast(t.value.server_tab.errors.no_selection);
    return;
  }
  soundManager.playClick();
  selectedTunnel.value = target;
  pendingDelete.value = { id: target.id, name: target.name };
  showDeleteModal.value = true;
};

const cancelDelete = () => {
  showDeleteModal.value = false;
  pendingDelete.value = null;
};

const confirmDeleteTunnel = async () => {
  const target = pendingDelete.value;
  if (!target) return;
  showDeleteModal.value = false;
  pendingDelete.value = null;

  // 隧道一删，列表里就查不到它绑过哪些域名了，而密码锁是按域名存的 ——
  // 所以先把「域名 + DNS 记录 ID」快照下来，后面补删域名、清密码锁都要靠它。
  const bound = collectBoundDomains(target.id);

  // 先停掉本机正在跑的那个进程：强制删除只是把隧道从 Cloudflare 侧摘掉，
  // 本机进程不主动停会一直重连报错，日志里刷屏。
  try {
    if (isTunnelRunning(target.name)) {
      await invoke<string>('stop_server_tunnel', { name: target.name });
      markServerStopped(target.name);
    }
  } catch {
    // 停不掉也不影响强制删除，继续往下走
  }

  const domainNote = bound.length ? `${fmt(t.value.logs.delete_bound_note, { count: bound.length })}` : '';
  appendLog(`[INFO] ${fmt(t.value.logs.force_deleting_tunnel, { name: target.name, note: domainNote })}...`, 'info', 'server');
  try {
    const res = await invoke<string>('delete_tunnel', { name: target.name });
    appendLog(`[SUCCESS] ${res}`, 'success', 'server');

    // 1. 域名：后端删隧道时会顺带清掉绑定的 CNAME，但拿不到 tunnel id 时会跳过，
    //    这里按快照核对一次，只补删真正还留在云端的那些。
    let dnsDeleted = 0;
    for (const d of await findLeftoverDomains(bound)) {
      try {
        await invoke<string>('delete_dns_route', { recordId: d.recordId });
        dnsDeleted += 1;
      } catch (err: any) {
        appendLog(`[WARN] ${fmt(t.value.logs.leftover_domain_cleanup_failed, { host: d.hostname, err: errorText(err) })}`, 'warn', 'server');
      }
    }
    if (dnsDeleted) appendLog(`[INFO] ${fmt(t.value.logs.leftover_dns_deleted, { count: dnsDeleted })}`, 'info', 'server');

    // 2. 密码锁：逐域名删掉云端的 Access 应用 / 策略 / Service Token（含本地记录），
    //    否则域名没了、锁还在云端拦着，软件里也再定位不到它。
    const { done, failed } = await purgeDomainLocks(bound);
    if (failed.length) {
      appendLog(
        `[WARN] ${fmt(t.value.logs.delete_lock_purge_failed, { list: failed.join(t.value.logs.cascade_join) })}`,
        'warn',
        'server',
      );
    }

    const noteParts: string[] = [];
    if (bound.length || dnsDeleted) noteParts.push(`${fmt(t.value.logs.cascade_domains, { count: Math.max(bound.length, dnsDeleted) })}`);
    if (done.length) noteParts.push(`${fmt(t.value.logs.cascade_locks, { count: done.length })}`);
    showToast(
      `${fmt(t.value.logs.tunnel_deleted, { name: target.name, note: noteParts.length ? fmt(t.value.logs.cascade_note, { list: noteParts.join(t.value.logs.cascade_join) }) : '' })}`,
    );

    if (selectedTunnel.value?.id === target.id) selectedTunnel.value = null;
    await handleRefreshTunnels();
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.tunnel_delete_failed, { err })}`, 'error', 'server');
  }
};

// 与后端对账客户端隧道：后端用 try_wait() 回收已退出的进程，返回真实在跑的快照；
// 顺手把「后端在跑但本地没保存过」的条目补进保存列表（旧版本遗留 / 外部启动），避免刷新后消失。
const refreshClientConnections = async () => {
  try {
    const list = await invoke<ClientTunnelItem[]>('list_client_tunnels');
    clientConnections.value = list;
    clientRunningKeys.value = list.map(x => x.key);

    const known = new Set(savedClientTunnels.value.map(t => t.key));
    let added = false;
    for (const item of list) {
      if (!known.has(item.key)) {
        savedClientTunnels.value.push({ key: item.key, domain: item.domain, port: item.port });
        known.add(item.key);
        added = true;
      }
    }
    if (added) persistClientTunnels();
  } catch {}
};

// 「刷新」按钮：与后端对账，并在控制台输出条数
const refreshClientTunnels = async (withLog = false) => {
  await refreshClientConnections();
  if (withLog) {
    appendLog(
      `[INFO] ${fmt(t.value.logs.client_list_refreshed, { count: clientRunningKeys.value.length })}`,
      'info',
      'client',
    );
  }
};

// 打开「添加客户端隧道」弹窗
const openClientAddModal = () => {
  soundManager.playClick();
  editingClientKey.value = '';
  clientForm.value.domain = '';
  clientForm.value.tokenId = '';
  clientForm.value.tokenSecret = '';
  clientFormDomainHasError.value = false;
  clientFormPortHasError.value = false;
  showClientAddModal.value = true;
};

// 打开「编辑客户端隧道」弹窗（运行中的条目不参与编辑，按钮已置灰）
const openClientEditModal = (row: SavedClientTunnel) => {
  soundManager.playClick();
  editingClientKey.value = row.key;
  clientForm.value.domain = row.domain;
  clientForm.value.port = row.port;
  clientForm.value.tokenId = row.tokenId || '';
  clientForm.value.tokenSecret = row.tokenSecret || '';
  clientFormDomainHasError.value = false;
  clientFormPortHasError.value = false;
  showClientAddModal.value = true;
};

const cancelClientAdd = () => {
  showClientAddModal.value = false;
  editingClientKey.value = '';
  clientFormDomainHasError.value = false;
  clientFormPortHasError.value = false;
};

// 保存客户端隧道：新增走「保存 + 自动启动」，编辑只改配置不动进程
const confirmClientAdd = async () => {
  if (clientSubmitting.value) return;

  const domain = clientForm.value.domain.trim();
  const port = clientForm.value.port.trim();
  // 访问凭据：去空格后成对保留（只填其一会先被 clientTokenHasError 拦下）
  const tokenId = clientForm.value.tokenId.trim();
  const tokenSecret = clientForm.value.tokenSecret.trim();

  if (!isDomainValid(domain)) {
    clientFormDomainHasError.value = true;
    appendLog(`[ERROR] ${t.value.logs.client_domain_invalid}`, 'error', 'client');
    return;
  }
  if (!isPortValid(port)) {
    clientFormPortHasError.value = true;
    appendLog(`[ERROR] ${t.value.logs.client_port_invalid}`, 'error', 'client');
    return;
  }
  if (tokenId.length > 0 !== tokenSecret.length > 0) {
    appendLog(`[ERROR] ${t.value.logs.client_token_pair_invalid}`, 'error', 'client');
    return;
  }

  const key = clientTunnelKey(domain, port);
  const entry: SavedClientTunnel = tokenId && tokenSecret
    ? { key, domain, port, tokenId, tokenSecret }
    : { key, domain, port };

  // 编辑模式：只改已保存的配置
  if (editingClientKey.value) {
    const dup = savedClientTunnels.value.some(
      t => t.key === key && t.key !== editingClientKey.value,
    );
    if (dup) {
      appendLog(`[ERROR] ${fmt(t.value.logs.client_dup_edit, { key })}`, 'error', 'client');
      showToast(t.value.logs.client_dup_toast);
      return;
    }
    const idx = savedClientTunnels.value.findIndex(t => t.key === editingClientKey.value);
    if (idx !== -1) savedClientTunnels.value[idx] = entry;
    persistClientTunnels();
    soundManager.playSuccess();
    appendLog(`[SUCCESS] ${fmt(t.value.logs.client_updated, { target: `${domain}:${port}` })}`, 'success', 'client');
    showToast(t.value.logs.client_saved);
    showClientAddModal.value = false;
    editingClientKey.value = '';
    return;
  }

  // 新增模式：重复直接拦掉，只落库保存，不自动启动（由用户点行内「启动」再连）
  if (savedClientTunnels.value.some(t => t.key === key)) {
    appendLog(`[ERROR] ${fmt(t.value.logs.client_dup_add, { key })}`, 'error', 'client');
    showToast(t.value.logs.client_dup_toast);
    return;
  }

  savedClientTunnels.value.push(entry);
  persistClientTunnels();
  soundManager.playSuccess();
  appendLog(`[SUCCESS] ${fmt(t.value.logs.client_created, { target: `${domain}:${port}` })}`, 'success', 'client');
  showToast(t.value.logs.client_saved_hint);
  showClientAddModal.value = false;
};

// 启动列表里某一条已保存的客户端隧道
const handleStartClient = async (row: SavedClientTunnel) => {
  soundManager.playClick();
  try {
    // 启动成功的日志由 Rust 侧统一广播，这里不再重复打印
    await invoke<string>('start_client_tunnel', {
      domain: row.domain,
      port: row.port,
      serviceTokenId: row.tokenId || null,
      serviceTokenSecret: row.tokenSecret || null,
    });
    showToast(`${fmt(t.value.logs.client_connected, { target: `${row.domain}:${row.port}` })}`);
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.client_connect_failed, { err })}`, 'error', 'client');
    showToast(`${err}`);
  } finally {
    await refreshClientConnections();
  }
};

// 断开指定的那一条客户端连接：条目保留在列表里，按钮转为「启动」(普通点击音效)
const handleStopClient = async (conn: SavedClientTunnel) => {
  soundManager.playClick();
  try {
    // 断开成功的日志由 Rust 侧统一广播，这里不再重复打印
    await invoke<string>('stop_client_tunnel', {
      domain: conn.domain,
      port: conn.port,
    });
    showToast(`${fmt(t.value.logs.client_disconnected, { target: `${conn.domain}:${conn.port}` })}`);
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.client_disconnect_failed, { err })}`, 'error', 'client');
  } finally {
    await refreshClientConnections();
  }
};

// 删除按钮：先弹二次确认，确认后才真正删除
const handleDeleteClient = (row: SavedClientTunnel) => {
  soundManager.playClick();
  pendingDeleteClient.value = row;
  showClientDeleteModal.value = true;
};

const cancelDeleteClient = () => {
  showClientDeleteModal.value = false;
  pendingDeleteClient.value = null;
};

// 确认删除：运行中的先断开，再从列表里移除（进程与配置一起清掉）
const confirmDeleteClient = async () => {
  const row = pendingDeleteClient.value;
  showClientDeleteModal.value = false;
  pendingDeleteClient.value = null;
  if (!row) return;
  if (isClientRunning(row.key)) {
    try {
      await invoke<string>('stop_client_tunnel', { domain: row.domain, port: row.port });
    } catch {}
  }
  savedClientTunnels.value = savedClientTunnels.value.filter(t => t.key !== row.key);
  persistClientTunnels();
  appendLog(`[INFO] ${fmt(t.value.logs.client_deleted, { target: `${row.domain}:${row.port}` })}`, 'warn', 'client');
  showToast(`${fmt(t.value.logs.client_deleted_toast, { target: `${row.domain}:${row.port}` })}`);
  await refreshClientConnections();
};

// 安装 cloudflared 流程 (根据系统与架构获取官方直链并下载至应用目录)
const handleInstallCloudflared = async () => {
  const target = getCloudflaredTarget();
  appendLog(`[INFO] ${fmt(t.value.logs.detected_env, { display: target.displayName, os: target.os, arch: target.arch })}`, 'info', 'misc');
  appendLog(`[INFO] ${fmt(t.value.logs.target_binary, { file: target.fileName })}`, 'info', 'misc');
  appendLog(`[INFO] ${fmt(t.value.logs.official_url, { url: target.downloadUrl })}`, 'info', 'misc');

  isDownloadingCloudflared.value = true;
  try {
    const res = await invoke<string>('download_and_install_cloudflared', {
      downloadUrl: target.downloadUrl,
      filename: target.fileName,
    });
    showToast(t.value.logs.download_started);
    appendLog(`[INFO] ${res}`, 'info', 'misc');
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.install_failed, { err })}`, 'error', 'misc');
  } finally {
    isDownloadingCloudflared.value = false;
  }
};

// 杂项操作
const handleCloudflaredLogin = async () => {
  try {
    await invoke<string>('login_cloudflared');
    showToast(t.value.logs.login_started);
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.login_failed, { err })}`, 'error', 'misc');
  }
};

const handleCheckVersion = async () => {
  try {
    const ver = await invoke<string>('check_cloudflared_version');
    appendLog(`[INFO] ${fmt(t.value.logs.version_current, { version: ver })}`, 'info', 'misc');
    showToast(`${fmt(t.value.logs.version_toast, { version: ver })}`);
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.version_check_failed, { err })}`, 'error', 'misc');
  }
};

const handleUpdateCloudflared = async () => {
  appendLog(`[INFO] ${t.value.logs.updating_cloudflared}`, 'info', 'misc');
  try {
    const res = await invoke<string>('update_cloudflared');
    appendLog(`[INFO] ${fmt(t.value.logs.update_result, { result: res })}`, 'info', 'misc');
    showToast(t.value.logs.update_done);
  } catch (err: any) {
    appendLog(`[ERROR] ${fmt(t.value.logs.update_failed, { err })}`, 'error', 'misc');
  }
};

const handleDownloadCloudflared = async () => {
  await openUrl('https://github.com/cloudflare/cloudflared/releases/latest');
};

// 真正退出程序
const confirmExitApp = async () => {
  showExitConfirmModal.value = false;
  try {
    await invoke('exit_app');
  } catch (e) {
    console.error('Exit app error:', e);
  }
};

// 键盘快捷键与全局点击监听 (ESC 关闭模态窗与放大预览)
const onKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    if (showDnsEditModal.value) {
      cancelEditDnsRoute();
    } else if (showDnsAddModal.value) {
      cancelDnsAdd();
    } else if (showDnsUnbindModal.value) {
      cancelUnbindDnsRoute();
    } else if (showClientAddModal.value) {
      cancelClientAdd();
    } else if (showClientDeleteModal.value) {
      cancelDeleteClient();
    } else if (showQuickStopModal.value) {
      cancelStopQuick();
    } else if (showDeleteModal.value) {
      showDeleteModal.value = false;
    } else if (showExitConfirmModal.value) {
      showExitConfirmModal.value = false;
    }
  }
};

// 初始化与事件监听
onMounted(async () => {
  syncThemeToDocument();
  document.title = t.value.title;
  window.addEventListener('keydown', onKeyDown);

  // 获取并监听窗口最大化状态
  try {
    isMaximized.value = await invoke<boolean>('is_window_maximized');
  } catch {}

  // 执行一次输入合法性初步检查（如有初始值）
  if (clientForm.value.port) onClientFormPortInput();

  // 监听 Rust 后端进程日志广播
  try {
    await listen<{ message: string; level: LogEntry['level']; source: LogEntry['source'] }>(
      'log-message',
      (event) => {
        appendLog(event.payload.message, event.payload.level, event.payload.source);
      }
    );

    // 监听系统托盘点击「退出程序」事件
    await listen('show-exit-confirm', () => {
      showExitConfirmModal.value = true;
    });

    // 监听临时链接临时域名分配
    await listen<{ key: string; url: string }>('quick-tunnel-url', (event) => {
      const { key, url } = event.payload;
      const item = quickTunnels.value.find(t => t.key === key);
      if (item) {
        item.url = url;
        item.status = 'running';
      } else {
        // 若列表里没有（如应用重连后），补一条
        const { protocol, port } = parseQuickKey(key);
        quickTunnels.value.push({ key, protocol, port, url, status: 'running' });
      }
      appendLog(`[SUCCESS] ${fmt(t.value.logs.quick_domain_assigned, { url })}`, 'success', 'quick');
      showToast(t.value.logs.quick_domain_ready);
    });
  } catch (e) {
    console.error('Listen event error:', e);
  }

  // 旧版本的云端托管 Token 是明文存在 localStorage 里的，现在改成每次用 cert.pem 现取，
  // 历史残留的键在这里清掉，免得一条再用不到的凭据继续躺在 WebView 数据目录里。
  try {
    localStorage.removeItem('remote_tunnels_v1');
    localStorage.removeItem('remote_token');
  } catch {}

  // 异步获取初始隧道列表与状态
  try {
    const serverKeys = await invoke<string[]>('is_server_running');
    serverRunningNames.value = serverKeys;
    await refreshClientConnections();
    const quickKeys = await invoke<string[]>('is_quick_running');
    quickTunnels.value = quickKeys.map(key => {
      const { protocol, port } = parseQuickKey(key);
      return {
        key,
        protocol,
        port,
        url: '',
        status: 'running' as const,
      };
    });
    await handleRefreshTunnels();
  } catch {}
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
});
</script>

<style scoped>
/* =========================================================================
   Windows 11 Fluent Design System 样式规范
   ========================================================================= */

.light-theme {
  --bg-app: #f3f3f3;
  --bg-card: rgba(255, 255, 255, 0.88);
  --bg-card-solid: #ffffff;
  --bg-input: #ffffff;
  --bg-tab-bar: #eaeaea;
  --bg-hover: rgba(0, 0, 0, 0.05);
  --bg-active: rgba(0, 0, 0, 0.08);
  --bg-table-header: #f8f9fa;
  --bg-table-row-hover: #f0f4f9;
  --bg-table-selected: #e0eef9;
  --text-primary: #1c1c1c;
  --text-secondary: #5c5c5c;
  --text-disabled: #8c8c8c;
  --border-subtle: rgba(0, 0, 0, 0.08);
  --border-strong: rgba(0, 0, 0, 0.16);
  --accent-color: #005fb8;
  --accent-hover: #006cdb;
  --accent-text: #ffffff;
  --danger-color: #c42b1c;
  --danger-hover: #b32617;
  --success-color: #107c10;
  --warning-color: #9d5d00;
  --error-border: #c42b1c;
  --error-bg: rgba(196, 43, 28, 0.06);
  --error-text: #c42b1c;
  /* 控制台（浅色主题 = 亮底黑字，跟随主题切换） */
  --console-bg: #ffffff;
  --console-text: #1c1c1c;
  --console-header-bg: #f3f3f3;
  --console-header-border: rgba(0, 0, 0, 0.08);
  --console-title-text: #1c1c1c;
  --console-count-text: #767676;
  --console-time-text: #6f6f6f;
  --console-empty-text: #8c8c8c;
  --console-btn-text: #3d3d3d;
  --console-btn-border: rgba(0, 0, 0, 0.16);
  --console-btn-hover-bg: rgba(0, 0, 0, 0.06);
  --console-btn-hover-text: #000000;
  --console-info-tag: #005fb8;
  --console-info-text: #1c1c1c;
  --console-warn-tag: #9d5d00;
  --console-warn-text: #7a4a00;
  --console-error-tag: #c42b1c;
  --console-error-text: #b02418;
  --console-success-tag: #107c10;
  --console-success-text: #0e6b0e;
  --shadow-card: 0 4px 12px rgba(0, 0, 0, 0.05), 0 1px 3px rgba(0, 0, 0, 0.03);
}

.dark-theme {
  --bg-app: #202020;
  --bg-card: rgba(38, 38, 38, 0.88);
  --bg-card-solid: #2b2b2b;
  --bg-input: #2c2c2c;
  --bg-tab-bar: #1c1c1c;
  --bg-hover: rgba(255, 255, 255, 0.07);
  --bg-active: rgba(255, 255, 255, 0.11);
  --bg-table-header: #282828;
  --bg-table-row-hover: #333333;
  --bg-table-selected: #1f3d59;
  --text-primary: #ffffff;
  --text-secondary: #b3b3b3;
  --text-disabled: #666666;
  --border-subtle: rgba(255, 255, 255, 0.08);
  --border-strong: rgba(255, 255, 255, 0.18);
  --accent-color: #60cdff;
  --accent-hover: #78d5ff;
  --accent-text: #000000;
  --danger-color: #ff99a4;
  --danger-hover: #ffb3bc;
  --success-color: #6ccb5f;
  --warning-color: #fce100;
  --error-border: #ff99a4;
  --error-bg: rgba(255, 153, 164, 0.12);
  --error-text: #ff99a4;
  /* 控制台（深色主题 = 终端风格深底） */
  --console-bg: #141414;
  --console-text: #d4d4d4;
  --console-header-bg: #1f1f1f;
  --console-header-border: rgba(255, 255, 255, 0.08);
  --console-title-text: #eaeaea;
  --console-count-text: #888888;
  --console-time-text: #777777;
  --console-empty-text: #666666;
  --console-btn-text: #cccccc;
  --console-btn-border: rgba(255, 255, 255, 0.15);
  --console-btn-hover-bg: rgba(255, 255, 255, 0.1);
  --console-btn-hover-text: #ffffff;
  --console-info-tag: #60cdff;
  --console-info-text: #d4d4d4;
  --console-warn-tag: #fce100;
  --console-warn-text: #ffe666;
  --console-error-tag: #ff99a4;
  --console-error-text: #ff99a4;
  --console-success-tag: #6ccb5f;
  --console-success-text: #98e68e;
  --shadow-card: 0 6px 16px rgba(0, 0, 0, 0.35);
}

/* 全局容器 (锁定全屏，无边框美学圆角) */
.fluent-window {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background-color: var(--bg-app);
  color: var(--text-primary);
  font-family: 'Segoe UI Variable Text', 'Segoe UI', -apple-system, BlinkMacSystemFont, Roboto, sans-serif;
  user-select: none;
  overflow: hidden;
  box-sizing: border-box;
  margin: 0;
  padding: 0;
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  transition: background-color 0.25s ease, color 0.25s ease;
}

.fluent-window.is-maximized {
  border-radius: 0;
  border: none;
}

/* 顶部标题栏 (最高层级 z-index: 5000, 完美支持拖拽与按钮对齐) */
.fluent-header {
  position: relative;
  z-index: 5000;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  height: 42px;
  background-color: var(--bg-card);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  cursor: default;
  user-select: none;
  -webkit-app-region: drag;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  pointer-events: auto;
}

/* Logo 专属悬浮放大、抖动动画与彩蛋响应 (pointer-events: auto, no-drag) */
.app-logo-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: transform 0.22s cubic-bezier(0.34, 1.56, 0.64, 1);
  user-select: none;
  cursor: pointer !important;
  pointer-events: auto !important;
  -webkit-app-region: no-drag !important;
  position: relative;
  z-index: 10;
}

.app-logo-wrapper:hover {
  transform: scale(1.22);
}

.app-logo-wrapper:active {
  transform: scale(0.92);
}

.app-logo-wrapper.shake {
  animation: logoShake 0.36s cubic-bezier(0.36, 0.07, 0.19, 0.97) both;
}

@keyframes logoShake {
  0%, 100% { transform: scale(1.22) rotate(0deg); }
  20% { transform: scale(1.18) rotate(-14deg); }
  40% { transform: scale(1.18) rotate(14deg); }
  60% { transform: scale(1.18) rotate(-8deg); }
  80% { transform: scale(1.18) rotate(8deg); }
}

.app-logo {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  filter: drop-shadow(0 2px 4px rgba(0,0,0,0.15));
  pointer-events: none;
}

.app-title-group,
.app-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.2px;
  pointer-events: none;
}

.header-center {
  flex: 1;
  height: 100%;
  pointer-events: none;
}

/* 顶部右侧所有操作与窗口控制按钮 (Flex 对齐、统一高度与边距、绝对置顶响应点击) */
.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 100%;
  pointer-events: auto !important;
  -webkit-app-region: no-drag !important;
  position: relative;
  z-index: 99999;
}

/* 统一顶部所有图标与窗口控制按钮的尺寸与内边距 (28px x 28px) */
.fluent-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  box-sizing: border-box;
  background-color: var(--bg-input);
  color: var(--text-primary);
  border: 1px solid var(--border-strong);
  border-radius: 6px;
  cursor: pointer !important;
  font-size: 13.5px;
  outline: none;
  transition: all 0.2s cubic-bezier(0.25, 1, 0.5, 1);
  user-select: none;
  pointer-events: auto !important;
  -webkit-app-region: no-drag !important;
}

.fluent-icon-btn:hover {
  background-color: var(--bg-hover);
  border-color: var(--text-secondary);
}

.fluent-icon-btn:active {
  transform: scale(0.92);
}

/* 窗口控制按钮专属样式 (与音效/主题按钮完全对齐) */
.win-ctrl-btn {
  color: var(--text-primary);
  pointer-events: auto !important;
  -webkit-app-region: no-drag !important;
}

.win-ctrl-btn.close-btn:hover {
  background-color: #c42b1c !important;
  color: #ffffff !important;
  border-color: #c42b1c !important;
}

.win-ctrl-btn.close-btn:active {
  background-color: #a82315 !important;
  color: #ffffff !important;
  border-color: #a82315 !important;
}

.win-ctrl-icon {
  display: block;
  pointer-events: none;
}

/* 音效切换按钮动效 */
.sound-btn {
  font-size: 14px;
}

.sound-icon {
  display: inline-block;
  transition: transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.sound-icon.spin {
  animation: soundBounce 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes soundBounce {
  0% { transform: scale(0.7); }
  50% { transform: scale(1.35); }
  100% { transform: scale(1); }
}

/* GitHub 仓库链接按钮 */
.github-btn {
  font-size: 14px;
}

.github-icon {
  display: block;
  transition: transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.github-btn:hover .github-icon {
  transform: scale(1.12);
}

/* 主题切换按钮动效 */
.theme-icon {
  display: inline-block;
  transition: transform 0.45s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.theme-icon.spin {
  animation: themeSpin 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes themeSpin {
  0% { transform: rotate(0deg) scale(0.8); }
  50% { transform: rotate(180deg) scale(1.25); }
  100% { transform: rotate(360deg) scale(1); }
}

/* 双栏布局：左侧侧边栏 + 主体内容区 */
.app-layout {
  position: relative;
  z-index: 5;
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}

/* 左侧侧边栏 (默认折叠为图标栏，可展开显示文字) */
.fluent-sidebar {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 178px;
  flex-shrink: 0;
  padding: 10px 8px;
  background-color: var(--bg-tab-bar);
  border-right: 1px solid var(--border-subtle);
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-width: thin;
  transition: width 0.22s cubic-bezier(0.25, 1, 0.5, 1),
              padding 0.22s cubic-bezier(0.25, 1, 0.5, 1);
}

.fluent-sidebar.collapsed {
  width: 56px;
  padding: 10px 6px;
}

/* 侧边栏展开/折叠按钮 */
.sidebar-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  height: 30px;
  margin-bottom: 6px;
  padding: 0 6px;
  background-color: var(--bg-input);
  color: var(--text-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: 6px;
  font-size: 12.5px;
  cursor: pointer;
  flex-shrink: 0;
  white-space: nowrap;
  overflow: hidden;
  transition: color 0.18s ease, background-color 0.18s ease;
}

.sidebar-toggle:hover {
  color: var(--text-primary);
  background-color: var(--bg-hover);
}

.toggle-icon {
  font-size: 13px;
  line-height: 1;
}

.toggle-text {
  font-size: 12px;
}

/* 一级菜单分组 */
.sidebar-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  padding: 8px 9px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  text-align: left;
  white-space: nowrap;
  cursor: pointer;
  transition: background-color 0.18s ease, color 0.18s ease;
}

.sidebar-item:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.sidebar-item.active {
  background-color: var(--bg-active);
  color: var(--text-primary);
  font-weight: 600;
}

/* 选中项左侧强调竖条 */
.sidebar-item.active::before {
  content: '';
  position: absolute;
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 16px;
  border-radius: 0 3px 3px 0;
  background-color: var(--accent-color);
}

.sidebar-icon {
  flex-shrink: 0;
  width: 18px;
  text-align: center;
  font-size: 14px;
}

.sidebar-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar-arrow {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--text-secondary);
  transition: transform 0.2s ease;
}

.sidebar-item.expanded .sidebar-arrow {
  transform: rotate(90deg);
}

/* 折叠态下强调竖条贴合边缘，避免被 overflow 裁掉 */
.fluent-sidebar.collapsed .sidebar-item.active::before {
  left: -6px;
}

/* 服务端下边栏 (子项列表) */
.sidebar-sub-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin: 2px 0 2px 10px;
  padding-left: 8px;
  border-left: 1px solid var(--border-subtle);
  overflow: hidden;
}

.sidebar-sub-item {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 100%;
  padding: 6px 8px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 5px;
  color: var(--text-secondary);
  font-size: 12.5px;
  text-align: left;
  white-space: nowrap;
  cursor: pointer;
  transition: background-color 0.18s ease, color 0.18s ease;
}

.sidebar-sub-item:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.sidebar-sub-item.active {
  background-color: var(--bg-active);
  color: var(--accent-color);
  font-weight: 600;
}

/* 下边栏展开/折叠过渡 */
.sub-list-enter-active,
.sub-list-leave-active {
  transition: opacity 0.18s ease, max-height 0.22s ease;
  max-height: 180px;
}

.sub-list-enter-from,
.sub-list-leave-to {
  opacity: 0;
  max-height: 0;
}

/* 侧边栏折叠态：只留图标 */
.fluent-sidebar.collapsed .sidebar-text,
.fluent-sidebar.collapsed .sidebar-arrow,
.fluent-sidebar.collapsed .toggle-text {
  display: none;
}

.fluent-sidebar.collapsed .sidebar-item,
.fluent-sidebar.collapsed .sidebar-sub-item {
  justify-content: center;
  padding-left: 0;
  padding-right: 0;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-dot.green {
  background-color: var(--success-color);
  box-shadow: 0 0 6px var(--success-color);
}

/* 主体内容区 (取消页面整体上下滚动) */
.fluent-body {
  position: relative;
  z-index: 5;
  flex: 1;
  min-width: 0;
  padding: 12px 16px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.tab-view {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
  overflow-y: auto;
  padding-right: 4px;
}

.animated-view {
  animation: tabViewFade 0.22s cubic-bezier(0.25, 1, 0.5, 1);
}

@keyframes tabViewFade {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

/* 卡片容器 */
.fluent-card {
  background-color: var(--bg-card);
  backdrop-filter: blur(16px);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  padding: 14px 16px;
  box-shadow: var(--shadow-card);
}

.form-card {
  flex-shrink: 0;
  /* 比 .fluent-card 的 14px 更紧凑：表单卡是高度预算里的最大头，
     收紧上下内边距能给隧道列表卡让出约 8px，默认窗口下多露一行隧道。 */
  padding-top: 10px;
  padding-bottom: 10px;
}

/* DNS 路由绑定卡片 (位于服务端本地视图)
   改两列：左列「绑定表单」，右列「已绑定域名」列表，两列各占一半宽度。
   纵向堆叠时该卡高达 394px，而卡片可用高度仅约 261px，必然被挤出视口看不到底；
   两列后可压到约 242px（实测），整卡与域名列表都能一屏看全。 */
.dns-route-card {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 10px;
  padding-bottom: 10px;
}

/* 标题行：标题居左，「添加」按钮居右，两端对齐 */
.dns-route-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.dns-route-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

/* 客户端卡片：标题行（标题 + 连接数状态 + 添加按钮）+ 通栏连接列表。
   客户端支持多开，每条隧道一个进程，因此改成"一行一条 + 逐条断开"的表结构。 */
.client-card {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 10px;
  padding-bottom: 10px;
}

/* 云端托管列表卡片：与客户端列表保持同一套版式 */
.remote-card {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 10px;
  padding-bottom: 10px;
}

.client-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.client-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.client-title-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

/* 连接列表：列表再长也在卡片内部滚动，不会把卡片撑出屏幕 */
.client-table-wrapper {
  width: 100%;
  max-height: 240px;
}

/* 本地端口列：等宽字体、不折行 */
.col-port {
  white-space: nowrap;
}

/* 「无需填写地址」的说明行（hello_world）。高度对齐旁边的 .fluent-input，
   纯文字、不画输入框边框 —— 避免看起来像个空输入框。 */
.address-hint {
  display: flex;
  align-items: center;
  min-height: 37px;
  font-size: 13px;
  color: var(--text-secondary);
}

/* 服务端页：让子视图参与父级的高度约束。
   约束生效后：三张卡片按可用高度分配，隧道列表在卡片内部滚动，
   DNS 路由卡始终完整可见；内容真超出时由外层 .tab-view 兜底滚动。
   注意：子视图可压缩（flex:1），所以压缩空间必须由各卡片自己的
   min-height 兜底，否则隧道列表会被一路压到只剩一条表头 —— 见下方
   .table-card > .fluent-table-wrapper 的 min-height。 */
.server-view .server-sub-view {
  flex: 1;
  min-height: 0;
}

/* 「已绑定域名」区块：按隧道分组的卡片列表（已取代原先的三列表格）。
   层级靠「容器底色 + 域名行左侧竖线」表达，不再依赖列宽对齐 —— 表头那三列已删，
   连带原先那套 .hostname-chip / .lock-chip-row / .chip-action-btn 胶囊样式一并废弃。 */
.dns-group-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  /* 分组多了在卡片内部滚动：列表再长也不会把卡片撑出视口。
     上限约 3 个域名区块（组头 28 + 3 × 50 ≈ 180），再高就会挤掉上方的隧道列表，
     所以宁可内部滚动 —— 隧道列表那边有 min-height: 150px 的地板，不能被无限压缩。 */
  max-height: 200px;
  overflow-y: auto;
  padding-right: 2px;
}

/* 一条隧道 = 一个分组容器。
   边框必须用 --border-strong：--border-subtle（0.08α）在浅色主题下实测几乎看不见，
   分组容器一消失，「哪个域名属于哪条隧道」这个层级就又回到了改版前的问题。 */
.dns-group {
  flex-shrink: 0;
  border: 1px solid var(--border-strong);
  border-radius: 8px;
  overflow: hidden;
}

/* 分组标题条：隧道名 + 域名数。用 --bg-table-header 铺浅底（浅色 #f8f9fa / 深色 #282828，
   两套主题都有值），跟下方域名区块分开，告诉人「这几个域名属于它」。 */
.dns-group-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 10px;
  background-color: var(--bg-table-header);
}

.dns-group-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.dns-group-count {
  flex-shrink: 0;
  font-size: 11.5px;
  color: var(--text-secondary);
}

/* 一个域名 = 组内一个区块，含两排：域名行 + 密码锁行。
   左侧 3px 竖线把所有域名的层级拉平，同时用颜色标示有没有上锁。 */
.dns-domain {
  padding: 5px 10px 6px 10px;
  border-top: 1px solid var(--border-subtle);
  border-left: 3px solid #b4b2a9;
}

.dns-domain:first-of-type {
  border-top: none;
}

.dns-domain.locked {
  border-left-color: #639922;
}

.dns-domain-row {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

/* 域名本身：等宽字体，点击即复制（沿用本面板的既有约定，不额外放复制按钮） */
.dns-domain-host {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  color: var(--text-primary);
  cursor: pointer;
  user-select: none;
}

.dns-domain-host:hover {
  text-decoration: underline;
}

/* 域名的密码锁设置行：标签 + 状态 + 凭据 + 操作。
   允许换行 —— 凭据展开成全文后一定放不下，折行而不是把行撑出卡片。 */
.dns-lock-row {
  flex-wrap: wrap;
  margin-top: 2px;
  row-gap: 3px;
}

.dns-lock-label {
  flex-shrink: 0;
  font-size: 11.5px;
  color: var(--text-secondary);
}

/* 锁状态徽标：已上锁用绿（保护生效中），未上锁用中性灰，不用红色 —— 未上锁是常态不是错误 */
.dns-lock-state {
  flex-shrink: 0;
  padding: 0 6px;
  border: 1px solid transparent;
  border-radius: 9px;
  font-size: 11px;
  white-space: nowrap;
}

.dns-lock-state.on {
  background-color: #EAF3DE;
  border-color: #97C459;
  color: #3B6D11;
}

.dns-lock-state.off {
  background-color: var(--bg-table-header);
  border-color: var(--border-strong);
  color: var(--text-secondary);
}

/* 行内操作按钮统一靠右：一个 flex 容器收口，避免给每个按钮都写 margin-left:auto
   （多个 auto 会平分空白，按钮被撒得满行都是）。窄窗口下按钮组自己换行，不挤出卡片。 */
.dns-row-actions {
  display: inline-flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 5px;
  flex-shrink: 0;
  margin-left: auto;
}

/* 凭据：行内只显示掩码，点击复制完整值。
   展开（显示凭据）后不走这里 —— 由下面的 .dns-cred-block 整行块承载 */
.dns-cred {
  flex-shrink: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 11.5px;
  color: var(--text-secondary);
  cursor: pointer;
  user-select: none;
}

.dns-cred:hover {
  text-decoration: underline;
}

/* 展开的凭据块：整行、账号与密码各一行（标签 + 值），缩进挂在域名竖线里侧。
   从锁行里挪出来独占整行，32 位账号 + 40 位密码就不会把锁行折成三四段、
   把「隐藏凭据 / 换密码 / 解锁」挤到最下面 —— 那是之前最难看的形态 */
.dns-cred-block {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 3px;
  padding-left: 8px;
  border-left: 2px solid var(--border-subtle);
}

.dns-cred-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
  min-width: 0;
}

.dns-cred-key {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--text-secondary);
}

/* 值：等宽、点击即复制。窄窗下允许在任意字符处折行，但绝不省略
   —— 凭据少一位就发不出去了 */
.dns-cred-val {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
  font-size: 11.5px;
  color: var(--text-primary);
  cursor: pointer;
  user-select: none;
}

.dns-cred-val:hover {
  text-decoration: underline;
}

/* 行内小按钮（改名 / 解绑 / 显示凭据 / 换密码 / 解锁 / 上锁）。
   一律用文字而不是 ✎ 🗑 🔁 🔓 图标：浅色底上 ✎ 是细线条字、🗑 是彩色 emoji，
   同一行里两种字形大小与质感都不一致（实测截图确认），文字按钮宽度统一、语义也更直白。 */
.dns-mini-btn {
  flex-shrink: 0;
  padding: 1px 8px;
  border: 1px solid var(--border-strong);
  border-radius: 4px;
  background-color: transparent;
  color: var(--text-primary);
  font-family: inherit;
  font-size: 11px;
  line-height: 1.5;
  white-space: nowrap;
  cursor: pointer;
}

/* 悬停统一走 --bg-hover：它两套主题都有值（浅色黑 5% / 深色白 7%），
   比写死 rgba 或浅色底稳 —— 深色主题下浅蓝底会糊成一片。 */
.dns-mini-btn:hover {
  background-color: var(--bg-hover);
}

/* 语义色只落在描边与文字上，且用主题变量而非写死色值（深色主题会换成浅色号） */
.dns-mini-btn.primary {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.dns-mini-btn.danger {
  border-color: var(--danger-color);
  color: var(--danger-color);
}

.dns-mini-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.dns-empty {
  padding: 14px 0;
  text-align: center;
  font-size: 12.5px;
  color: var(--text-secondary);
}

/* 解绑按钮：危险色描边，与行内启停按钮区分 */
.row-action-btn.danger {
  border-color: var(--danger-color);
}

.row-action-btn.danger:hover {
  background-color: var(--danger-color);
  border-color: var(--danger-color);
}

/* 弹窗内的上下文说明行（隧道 · 域名） */
.modal-context {
  margin: 0 0 12px;
  font-size: 12.5px;
  font-family: 'Consolas', 'Courier New', monospace;
  color: var(--text-secondary);
  word-break: break-all;
}

/* 删除确认弹窗里的「强制删除」说明：明确告诉用户即使隧道在跑也会被删掉 */
.modal-danger-hint {
  margin: 8px 0 0;
  font-size: 12px;
  line-height: 1.6;
  color: var(--danger-color);
}

/* 服务端子视图状态圆点（侧边栏下边栏使用） */
.mode-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
  flex-shrink: 0;
}

.mode-dot.local {
  background-color: #639922;
}

.mode-dot.remote {
  background-color: #534AB7;
}

.mode-dot.quick {
  background-color: #f2a900;
}

/* 子视图容器 */
.server-sub-view {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 隧道类型徽章 */
.type-badge {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
}

.type-local {
  background-color: #EAF3DE;
  color: #3B6D11;
}

.type-remote {
  background-color: #EEEDFE;
  color: #534AB7;
}

/* 云端 ingress 配置展示 */
.remote-config-card {
  flex-shrink: 0;
}

.remote-config-body {
  background-color: var(--bg-input);
  border: 1px solid var(--border-strong);
  border-radius: 8px;
  padding: 12px;
  /* 多条云端隧道并行时配置会很长，这里封顶 + 内部滚动，避免卡片无限长高 */
  max-height: 360px;
  overflow-y: auto;
}

/* 配置按隧道分组：每条隧道一块，块间用细线分隔，避免多条隧道时糊成一片 */
.remote-config-group + .remote-config-group {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed var(--border-subtle);
}

/* 分组标题现在是隧道名称（隧道 ID 只留在 tooltip 里），所以不再用等宽字体 */
.remote-config-group-title {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.remote-config-group-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 每条隧道下三块：已发布应用程序路由 / 主机名路由 / CIDR 路由。
   三块来自面板上三个独立页面、三个不同接口，分开列清楚，别让人误以为是一套数据 */
.remote-config-section + .remote-config-section {
  margin-top: 8px;
}

.remote-config-section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 2px;
}

/* 空与「读不到」要分开显示：空是有数据源、就 0 条；失败是压根没读到 */
.remote-config-section-empty {
  font-size: 12px;
  color: var(--text-secondary);
  opacity: 0.7;
}

.remote-config-section-error {
  font-size: 12px;
  color: var(--danger-color);
  word-break: break-all;
}

.remote-config-body pre {
  margin: 0;
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.7;
}

.remote-config-empty {
  font-size: 12px;
  color: var(--text-secondary);
}

.card-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 10px;
}

.card-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
}

.card-subtitle {
  font-size: 12px;
  color: var(--text-secondary);
}

/* 表单与输入框 */
.form-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 14px;
  margin-bottom: 8px;
}

.fluent-form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.required {
  color: var(--danger-color);
}

.input-container {
  position: relative;
}

.fluent-input {
  width: 100%;
  padding: 8px 12px;
  background-color: var(--bg-input);
  color: var(--text-primary);
  border: 1px solid var(--border-strong);
  border-bottom: 2px solid var(--border-strong);
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
  transition: all 0.2s ease;
}

.fluent-input:focus {
  border-color: var(--accent-color);
  border-bottom-color: var(--accent-color);
  box-shadow: 0 0 0 1px var(--accent-color);
}

/* 核心输入错误标红样式 */
.fluent-input.input-error {
  border-color: var(--error-border) !important;
  border-bottom-color: var(--error-border) !important;
  background-color: var(--error-bg) !important;
  box-shadow: 0 0 0 1px var(--error-border) !important;
  color: var(--error-text) !important;
}

/* 下拉选择框 */
.fluent-select {
  appearance: none;
  -webkit-appearance: none;
  cursor: pointer;
  padding-right: 28px;
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2'><path d='M6 9l6 6 6-6'/></svg>");
  background-repeat: no-repeat;
  background-position: right 10px center;
}

.fluent-select option {
  background-color: var(--bg-input);
  color: var(--text-primary);
}

/* 字段提示文字 */
.field-hint {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 4px;
  line-height: 1.4;
}

.error-tip {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--error-text);
  margin-top: 2px;
  font-weight: 500;
  animation: shake 0.25s ease-in-out;
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-4px); }
  75% { transform: translateX(4px); }
}

/* 按钮与居中规范 */
.actions-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.center-actions {
  justify-content: center !important;
  width: 100%;
}

.fluent-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 18px;
  background-color: var(--bg-input);
  color: var(--text-primary);
  border: 1px solid var(--border-strong);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  outline: none;
  transition: all 0.15s ease;
}

.fluent-btn:hover:not(:disabled) {
  background-color: var(--bg-hover);
  border-color: var(--text-secondary);
}

.fluent-btn:active:not(:disabled) {
  background-color: var(--bg-active);
  transform: scale(0.98);
}

.fluent-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 刷新进行中，图标转起来。
   「按钮变灰、界面纹丝不动」会被当成按钮坏了 —— 加个转圈明确表达「正在忙」，
   尤其是在等网络请求、可能要十几秒才有结果的时候。
   .btn-icon 本身是 inline 元素，不设 inline-block 的话 transform 不生效。 */
.btn-icon.spinning {
  display: inline-block;
  animation: btnSpin 0.9s linear infinite;
}

@keyframes btnSpin {
  to {
    transform: rotate(360deg);
  }
}

.fluent-btn.primary {
  background-color: var(--accent-color);
  color: var(--accent-text);
  border-color: var(--accent-color);
}

.fluent-btn.primary:hover:not(:disabled) {
  background-color: var(--accent-hover);
  border-color: var(--accent-hover);
}

.fluent-btn.danger {
  background-color: var(--danger-color);
  color: #ffffff;
  border-color: var(--danger-color);
}

.fluent-btn.danger-outline {
  color: var(--danger-color);
  border-color: var(--danger-color);
}

.fluent-btn.danger-outline:hover:not(:disabled) {
  background-color: var(--error-bg);
}

.fluent-btn.large {
  padding: 8px 26px;
  font-size: 13.5px;
}

/* 运行状态指示徽章 */
.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid var(--border-subtle);
}

.status-pill.online {
  background-color: rgba(16, 124, 16, 0.12);
  color: var(--success-color);
  border-color: var(--success-color);
}

.status-pill.online .pill-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background-color: var(--success-color);
  box-shadow: 0 0 6px var(--success-color);
}

.status-pill.offline {
  background-color: var(--bg-hover);
  color: var(--text-disabled);
}

.status-pill.offline .pill-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background-color: var(--text-disabled);
}

/* 数据表格 (自适应填满剩余空间，带内部上下滑动 Slider) */
.table-card {
  display: flex;
  flex-direction: column;
  flex: 1 1 auto;
/* 这里必须给一个显式下限，不能写 min-height:0 / auto：
   CSS 规定 overflow 不是 visible 的元素，其 flex 自动最小尺寸为 0，
   于是窗口一矮这张卡会被一路压扁（实测 580 高时只剩 36px），
   里面 min-height:100px 的表格框随即溢出卡片被 overflow 切掉 ——
   看起来正是"隧道列表内容被挤压看不到了"。
   261px = 标题(36) + 表格框下限(152=150+2边框) + 按钮行(35+10margin)
           + 卡片上下内边距(28)。实测（真实 Chromium 渲染）：
           min-height 低于 261 时刷新/删除按钮被 overflow:hidden 裁掉
   —— 740 窗口卡片分到 226px，按钮整体不可见；800 窗口也裁 17px。
   压缩止步于此，再矮就交给外层 .tab-view 滚动，按钮行永远完整。 */
  min-height: 261px;
  overflow: hidden;
}

/* 隧道列表的表格框给一个高度下限：表头 + 2~3 整行。
   没有这个下限时，flex 压缩会把它压到只剩一条 sticky 表头，
   数据行全被顶出可视区 —— 这是"隧道列表内容消失"的真正原因。
   150px ≈ 表头 34 + 3 行 × 35 + 横向滚动条，保证默认窗口能直接看到至少 3 条隧道。 */
.table-card > .fluent-table-wrapper {
  min-height: 150px;
}

.fluent-table-wrapper {
  flex: 1;
  border: 1px solid var(--border-subtle);
  border-radius: 6px;
  overflow-y: auto !important;
  overflow-x: auto;
  background-color: var(--bg-card-solid);
}

.fluent-table {
  width: 100%;
  /* 内容过宽（如超长域名）时按内容撑开，由列表容器整体横向滚动 */
  min-width: max-content;
  border-collapse: collapse;
  text-align: left;
  font-size: 12.5px;
}

.fluent-table th {
  position: sticky;
  top: 0;
  background-color: var(--bg-table-header);
  padding: 8px 12px;
  font-weight: 600;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border-subtle);
  z-index: 2;
}

.fluent-table td {
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-subtle);
  white-space: nowrap;
  user-select: text !important;
}

.fluent-table tbody tr {
  cursor: pointer;
  transition: background-color 0.15s;
}

.fluent-table tbody tr:hover {
  background-color: var(--bg-table-row-hover);
}

.fluent-table tbody tr.selected {
  background-color: var(--bg-table-selected);
  font-weight: 500;
}

.mono {
  font-family: 'Consolas', 'Courier New', monospace;
}

.font-bold {
  font-weight: 600;
}

.empty-table {
  text-align: center;
  padding: 24px;
  color: var(--text-disabled);
}

/* 绑定域名列：域名过长时由列表整体横向滚动，不竖向折行 */
.col-hostname {
  white-space: nowrap;
}

/* 运行状态列（多开时逐条显示哪条在跑） */
.col-status {
  white-space: nowrap;
}

.tunnel-status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-disabled);
}

.tunnel-status.online {
  color: var(--success-color);
}

.status-dot.gray {
  background-color: var(--text-disabled);
  box-shadow: none;
}

/* 行内启停按钮（多开时逐条控制） */
.col-actions {
  white-space: nowrap;
}

.row-action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 22px;
  padding: 0;
  background: transparent;
  border: 1px solid var(--border-subtle);
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  line-height: 1;
  color: var(--text-primary);
  transition: background-color 0.15s ease, border-color 0.15s ease;
}

/* 一行里并排多个操作按钮时的间距 */
.col-actions .row-action-btn + .row-action-btn {
  margin-left: 4px;
}

.row-action-btn.primary {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.row-action-btn.primary:hover {
  background-color: var(--accent-color);
  border-color: var(--accent-color);
  color: #ffffff;
}

.row-action-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.row-action-btn:disabled:hover {
  background-color: transparent;
  border-color: var(--border-subtle);
}

.row-action-btn:hover {
  background-color: var(--bg-hover);
  border-color: var(--text-secondary);
}

.row-action-btn:active {
  background-color: var(--bg-active);
}

.icon-triangle {
  width: 0;
  height: 0;
  margin-left: 2px;
  border-left: 7px solid var(--accent-color);
  border-top: 4.5px solid transparent;
  border-bottom: 4.5px solid transparent;
}

.icon-square {
  width: 8px;
  height: 8px;
  border-radius: 1px;
  background-color: var(--danger-color);
}

.hostname-tag {
  display: inline-block;
  padding: 2px 8px;
  margin: 1px 2px;
  border-radius: 4px;
  background-color: #005fb8;
  color: #ffffff;
  font-size: 11.5px;
  font-family: 'Consolas', 'Courier New', monospace;
  white-space: nowrap;
  /* 点击即复制，给出手型光标与反馈 */
  cursor: pointer;
  user-select: none;
  transition: background-color 0.15s ease;
}

.hostname-tag:hover {
  background-color: #0078d4;
}

.hostname-tag:active {
  background-color: #004578;
}

/* ============ 隧道密码锁（Cloudflare Access） ============ */

/* 访问密码列：锁图标 + 明文凭据（点击复制），行宽不够时横向滚动 */
.col-password {
  min-width: 180px;
  max-width: 260px;
}

.lock-cell {
  display: flex;
  align-items: flex-start;
  gap: 6px;
}

.lock-btn {
  font-size: 12px;
  flex-shrink: 0;
}

/* 明文凭据：账号 / 密码各一行，小号等宽字体，溢出省略（点击复制全文） */
.lock-cred {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.lock-cred-line {
  font-size: 10.5px;
  line-height: 1.3;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 210px;
  cursor: pointer;
  user-select: none;
}

.lock-cred-line:hover {
  color: var(--text-primary);
}

/* 弹窗里的凭据输入框：点击即复制（无独立复制按钮），鼠标呈手型提示可点 */
.click-copy {
  cursor: pointer;
  user-select: all;
}

.click-copy:hover {
  border-color: var(--accent, #0078d4);
}

.lock-cred-empty {
  color: var(--text-disabled);
  font-size: 12px;
}

/* 密码锁开关（创建 / 修改弹窗里的一行式 checkbox） */
.lock-switch-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  margin-top: 4px;
  border: 1px solid var(--border-subtle);
  border-radius: 6px;
  cursor: pointer;
  user-select: none;
  transition: border-color 0.15s ease, background-color 0.15s ease;
}

.lock-switch-row:hover {
  background-color: var(--bg-hover);
}

.lock-switch-row.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.lock-switch-row input[type='checkbox'] {
  width: 15px;
  height: 15px;
  accent-color: var(--accent-color);
  cursor: pointer;
}

.lock-switch-icon {
  font-size: 14px;
}

.lock-switch-text {
  font-size: 12.5px;
  color: var(--text-primary);
}

/* 弹窗底部提示行（普通 / 警告两种色） */
.modal-hint {
  margin-top: 8px;
  font-size: 11.5px;
  line-height: 1.5;
  color: var(--text-secondary);
}

.modal-hint.warn {
  color: var(--warning-color, #c78a1d);
}

/* ============ 「创建 / 修改隧道」弹窗：三块云端配置编辑器 ============ */

/* 比通用弹窗宽：一行里要摆下「协议下拉 + 端口 + 域名 + 删除」四件东西。
   必须写成两级选择器 —— .fluent-modal-dialog 的 max-width:420px 在样式表里排在后面，
   同权重下后者胜出，只写 .tunnel-form-dialog 会被它压回 420px（实测就是被压住了）。 */
.fluent-modal-dialog.tunnel-form-dialog {
  max-width: 620px;
}

/* 三块编辑区加起来比一屏长，让内容区自己滚动，标题与底部按钮始终可见 */
.tunnel-form-body {
  max-height: 62vh;
  overflow-y: auto;
  /* 给滚动条留位，否则「＋ 添加路由」这类右对齐按钮会贴着滚动条 */
  padding-right: 8px;
}

/* 修改态下的隧道名：只读，但保持输入框的体量，方便跟创建态对照 */
.tunnel-form-name {
  padding: 8px 12px;
  border: 1px dashed var(--border-strong);
  border-radius: 4px;
  background-color: var(--bg-input);
  color: var(--text-secondary);
  font-size: 13px;
  word-break: break-all;
}

.form-hint {
  font-size: 11.5px;
  line-height: 1.5;
  color: var(--text-secondary);
}

.route-loading {
  margin-bottom: 12px;
  padding: 8px 10px;
  border-radius: 6px;
  background-color: var(--bg-input);
  font-size: 12px;
  color: var(--text-secondary);
}

/* 一块配置区：标题行 + 内容，用一条上边框跟上一块分开 */
.route-section {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid var(--border-subtle);
}

.route-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 8px;
}

.route-section-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

/* 一条 ingress 规则 = 一行：协议 / 端口 / 域名 / 删除 */
.ingress-row {
  margin-bottom: 8px;
}

.ingress-row-line {
  display: flex;
  align-items: flex-end;
  gap: 8px;
}

.ingress-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.ingress-field-label {
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
}

/* 都能收缩（flex-shrink: 1），窄窗口下三格一起让位，不会把整行顶出弹窗。
   子元素一律 min-width:0 —— <select> 的 min-content 是它最长那个选项的文字宽度，
   不归零的话 flex 收缩算不下去，行照样溢出。 */
.ingress-field > * {
  min-width: 0;
}

.ingress-field.protocol {
  flex: 0 1 180px;
}

.ingress-field.port {
  flex: 0 1 108px;
}

/* 域名（或 raw 的 service 原文）那一格吃掉剩下的宽度 */
.ingress-field.hostname,
.ingress-field.service {
  flex: 2 1 130px;
}

.ingress-remove {
  flex-shrink: 0;
  margin-bottom: 4px;
}

/* 末尾兜底规则行：它不靠域名匹配（是 ingress 的结构性末条），
   用虚线框跟上面按域名匹配的普通行区分开，也不给它删除按钮。
   整块只占一行高 —— 标签并进「协议」那一格的标签行、说明挪到下拉右侧，
   padding 也收紧到刚好包住这一行 */
.ingress-row.is-catch-all {
  margin-top: 8px;
  padding: 4px 10px 5px 10px;
  border: 1px dashed var(--border-strong);
  border-radius: 6px;
  background-color: var(--bg-input);
}

/* 「默认兜底」标签：并进协议那一格的标签行里（inline-block 才吃得到水平 padding） */
.catch-all-tag {
  display: inline-block;
  margin-right: 5px;
  padding: 0 6px;
  border-radius: 3px;
  font-size: 11px;
  background-color: var(--border-strong);
  color: var(--text-primary);
}

.catch-all-service {
  font-size: 11.5px;
  color: var(--text-secondary);
}

/* 兜底行的说明：跟协议下拉同排、吃掉剩余宽度。窄窗放不下就用省略号收尾，
   绝不许折成第二行 —— 一折整块又变成两行高了 */
.catch-all-hint {
  flex: 1 1 auto;
  min-width: 0;
  align-self: flex-end;
  margin-bottom: 7px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  font-size: 11px;
  color: var(--text-secondary);
}

/* 主机名 / CIDR 路由：一行 = 值 + 备注 + 删除 */
.route-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.route-row .fluent-input {
  min-width: 0;
}

/* 值那格固定一些、备注那格弹性，窄窗口下先压备注 */
.route-row .fluent-input:first-child {
  flex: 0 1 220px;
}

.route-row .fluent-input:nth-child(2) {
  flex: 1 1 120px;
}

.route-empty {
  font-size: 11.5px;
  line-height: 1.5;
  color: var(--text-secondary);
}

/* 凭据展示弹窗里的行内复制按钮 */
.copy-inline {
  flex-shrink: 0;
  margin-left: 6px;
}

/* ============ 配置页：Access Token 卡片 ============ */

.access-token-card {
  margin-bottom: 12px;
}

.access-token-row {
  display: flex;
  align-items: flex-end;
  gap: 10px;
}

.access-token-field {
  flex: 1;
  margin-bottom: 0;
}

.access-token-save {
  height: 34px;
  flex-shrink: 0;
}

.hostname-empty {
  color: var(--text-disabled);
  font-size: 12px;
}

/* 临时链接（临时域名）结果展示 */
.quick-url-box {
  margin-top: 14px;
  padding: 14px;
  border: 1px dashed var(--border-strong);
  border-radius: 8px;
  background-color: var(--bg-input);
}

.quick-url-box.active {
  border-style: solid;
  border-color: var(--success-color);
}

.quick-url-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.quick-url-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--accent-color);
  word-break: break-all;
  padding: 6px 0;
  /* 点击即复制 */
  cursor: pointer;
  user-select: none;
}

.quick-url-value:hover {
  text-decoration: underline;
}

.quick-url-empty {
  color: var(--text-disabled);
  font-size: 13px;
  padding: 6px 0;
}

.quick-url-actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}

.quick-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.quick-list-empty {
  padding: 16px;
  text-align: center;
  color: var(--text-disabled);
  font-size: 13px;
  border: 1px dashed var(--border-strong);
  border-radius: 8px;
}

.quick-item {
  padding: 12px;
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background-color: var(--bg-input);
}

.quick-item-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.quick-item-target {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.quick-item-url {
  min-height: 24px;
}

.quick-item-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.fluent-btn.small {
  padding: 5px 12px;
  font-size: 12px;
}

.remote-tunnel-card {
  margin-top: 12px;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--text-primary);
}

.hint-text {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.table-actions {
  margin-top: 10px;
  justify-content: flex-end;
  flex-shrink: 0;
}

/* 杂项网格磁贴 */
.tile-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
}

.tile-btn {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background-color: var(--bg-card-solid);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  cursor: pointer;
  text-align: left;
  transition: all 0.2s ease;
}

.tile-btn:hover {
  background-color: var(--bg-hover);
  border-color: var(--accent-color);
  transform: translateY(-2px);
  box-shadow: var(--shadow-card);
}

/* 「安装 cloudflared」「打开配置目录」两块磁贴不再做特殊描边：
   原来写的 border-color 是主题强调色（浅色 #005fb8 / 深色 #60cdff），
   那是一条**常驻**蓝边、不是 focus 环，看上去像一直在亮，用户明确要求去掉。
   现在两块与其它磁贴完全一致，只在 hover 时才有反馈。 */

.tile-icon {
  font-size: 24px;
}

.tile-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.tile-title {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.tile-desc {
  font-size: 11.5px;
  color: var(--text-secondary);
}

/* 醒目的安全防泄露警告横幅 (红色/危险强调色) */
.config-warning-banner {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-top: 12px;
  padding: 10px 14px;
  background-color: var(--error-bg);
  border: 1px solid var(--error-border);
  border-left: 4px solid var(--danger-color);
  border-radius: 6px;
  color: var(--danger-color);
  font-size: 12.5px;
  font-weight: 600;
  line-height: 1.5;
  animation: fadeIn 0.25s ease;
}

.config-warning-banner .warning-icon {
  font-size: 15px;
  flex-shrink: 0;
}

.config-warning-banner .warning-text {
  user-select: text !important;
}

/* 底部 Windows Terminal 控制台 (支持边缘拖拽调高 & 文本鼠标选中复制) */
.fluent-console {
  position: relative;
  display: flex;
  flex-direction: column;
  background-color: var(--console-bg);
  border-top: 1px solid var(--border-strong);
  flex-shrink: 0;
}

/* 控制台顶部调节把手 */
.console-resizer {
  position: absolute;
  top: -4px;
  left: 0;
  right: 0;
  height: 8px;
  cursor: ns-resize;
  z-index: 100;
}

.console-resizer:hover,
.console-resizer:active {
  background-color: var(--accent-color);
  height: 3px;
  top: -1px;
}

.console-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 14px;
  background-color: var(--console-header-bg);
  border-bottom: 1px solid var(--console-header-border);
  user-select: none;
}

.console-title-area {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--console-title-text);
}

.log-count {
  font-size: 11px;
  color: var(--console-count-text);
}

.console-actions {
  display: flex;
  gap: 8px;
}

.console-btn {
  padding: 2px 8px;
  background: transparent;
  color: var(--console-btn-text);
  border: 1px solid var(--console-btn-border);
  border-radius: 4px;
  font-size: 11px;
  cursor: pointer;
  user-select: none;
}

.console-btn:hover {
  background-color: var(--console-btn-hover-bg);
  color: var(--console-btn-hover-text);
}

/* 日志内容区 (完全允许鼠标划选与复制) */
.console-body {
  flex: 1;
  padding: 8px 14px;
  overflow-y: auto !important;
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.55;
  color: var(--console-text);
  box-sizing: border-box;
  user-select: text !important;
  -webkit-user-select: text !important;
  cursor: text;
}

.console-empty {
  color: var(--console-empty-text);
  font-style: italic;
  padding-top: 4px;
  user-select: none;
}

.console-line {
  display: flex;
  gap: 8px;
  word-break: break-all;
  user-select: text !important;
  -webkit-user-select: text !important;
}

.log-time {
  color: var(--console-time-text);
  flex-shrink: 0;
  user-select: text !important;
}

.log-tag {
  font-weight: 600;
  flex-shrink: 0;
  user-select: text !important;
}

.log-msg {
  user-select: text !important;
}

.log-info .log-tag { color: var(--console-info-tag); }
.log-info .log-msg { color: var(--console-info-text); }

.log-warn .log-tag { color: var(--console-warn-tag); }
.log-warn .log-msg { color: var(--console-warn-text); }

.log-error .log-tag { color: var(--console-error-tag); }
.log-error .log-msg { color: var(--console-error-text); }

.log-success .log-tag { color: var(--console-success-tag); }
.log-success .log-msg { color: var(--console-success-text); }

/* 模态对话框 */
.fluent-modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.fluent-modal-dialog {
  width: 90%;
  max-width: 420px;
  background-color: var(--bg-card-solid);
  border: 1px solid var(--border-strong);
  border-radius: 8px;
  box-shadow: 0 16px 36px rgba(0, 0, 0, 0.28);
  padding: 20px;
  box-sizing: border-box;
  animation: modalPop 0.2s cubic-bezier(0.1, 0.9, 0.2, 1);
}

@keyframes modalPop {
  from { opacity: 0; transform: scale(0.92); }
  to { opacity: 1; transform: scale(1); }
}

.modal-header {
  margin-bottom: 12px;
}

.modal-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
}

.modal-body {
  font-size: 13.5px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 20px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* Toast 提示 */
.fluent-toast {
  position: fixed;
  bottom: 190px;
  left: 50%;
  transform: translateX(-50%);
  background-color: var(--bg-card-solid);
  color: var(--text-primary);
  border: 1px solid var(--border-strong);
  border-radius: 20px;
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  z-index: 30000;
}

.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: all 0.25s ease;
}

.toast-fade-enter-from,
.toast-fade-leave-to {
  opacity: 0;
  transform: translate(-50%, 10px);
}
</style>
