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
          title="Cloudflared Logo（点击触发酒狐彩蛋语音~）"
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
          :title="isSoundEnabled ? '点击关闭/静音全局音效' : '点击开启全局音效'"
        >
          <span :class="['sound-icon', { spin: isSoundSpinning }]">
            {{ isSoundEnabled ? '🔊' : '🔇' }}
          </span>
        </button>

        <!-- 2. GitHub 仓库链接按钮 (指向 bayueqi/ZQ-CFTunnel) -->
        <button
          class="fluent-icon-btn github-btn"
          @click="openUrl('https://github.com/bayueqi/ZQ-CFTunnel')"
          title="前往 GitHub 仓库 (bayueqi/ZQ-CFTunnel)"
        >
          <svg class="github-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path fill="currentColor" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z"/>
          </svg>
        </button>

        <!-- 3. 语言选择器 -->
        <div class="fluent-dropdown-wrapper" ref="langDropdownRef">
          <button
            class="fluent-dropdown-btn"
            :class="{ open: isLangDropdownOpen }"
            @click="isLangDropdownOpen = !isLangDropdownOpen"
            type="button"
          >
            <span class="dropdown-selected-label">
              {{ currentLangItem?.flag }} {{ currentLangItem?.label }}
            </span>
            <span class="dropdown-arrow" :class="{ rotated: isLangDropdownOpen }">▾</span>
          </button>

          <transition name="dropdown-slide">
            <div v-if="isLangDropdownOpen" class="fluent-dropdown-menu">
              <div
                v-for="item in LANG_ORDER"
                :key="item.key"
                :class="['fluent-dropdown-item', { active: currentLang === item.key }]"
                @click="selectLanguage(item.key)"
              >
                <span class="item-flag">{{ item.flag }}</span>
                <span class="item-label">{{ item.label }}</span>
                <span v-if="currentLang === item.key" class="item-check">✓</span>
              </div>
            </div>
          </transition>
        </div>

        <!-- 4. 主题切换按钮 -->
        <button
          class="fluent-icon-btn theme-btn"
          @click="toggleTheme"
          :title="isDarkMode ? '切换到浅色模式' : '切换到深色模式'"
        >
          <span :class="['theme-icon', { spin: isThemeSpinning }]">
            {{ isDarkMode ? '☀' : '🌙' }}
          </span>
        </button>

        <!-- 5. 最小化按钮 (统一 28x28 尺寸，通过后端原生命令可靠最小化) -->
        <button
          class="fluent-icon-btn win-ctrl-btn minimize-btn"
          @click.stop="handleMinimize"
          title="最小化"
        >
          <svg class="win-ctrl-icon" width="10" height="1" viewBox="0 0 10 1">
            <rect width="10" height="1" fill="currentColor" />
          </svg>
        </button>

        <!-- 6. 最大化 / 还原按钮 (统一 28x28 尺寸，通过后端原生命令可靠缩放) -->
        <button
          class="fluent-icon-btn win-ctrl-btn maximize-btn"
          @click.stop="handleToggleMaximize"
          :title="isMaximized ? '还原' : '最大化'"
        >
          <svg v-if="!isMaximized" class="win-ctrl-icon" width="10" height="10" viewBox="0 0 10 10">
            <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1" />
          </svg>
          <svg v-else class="win-ctrl-icon" width="10" height="10" viewBox="0 0 10 10">
            <path d="M2.5 0.5H9.5V7.5M0.5 2.5H7.5V9.5H0.5Z" fill="none" stroke="currentColor" stroke-width="1" />
          </svg>
        </button>

        <!-- 7. 关闭按钮 (统一 28x28 尺寸，通过后端原生命令优雅隐藏到系统托盘) -->
        <button
          class="fluent-icon-btn win-ctrl-btn close-btn"
          @click.stop="handleCloseWindow"
          title="关闭窗口 (自动隐藏到系统托盘)"
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
          :title="sidebarCollapsed ? '展开侧边栏' : '折叠侧边栏'"
        >
          <span class="toggle-icon">{{ sidebarCollapsed ? '☰' : '⟨⟨' }}</span>
          <span v-if="!sidebarCollapsed" class="toggle-text">收起</span>
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

              <button
                :class="['sidebar-sub-item', { active: currentTab === 'server' && isServerView('remote') }]"
                @click="switchServerView('remote')"
                :title="t.server_tab.nav_remote"
              >
                <span class="mode-dot remote"></span>
                <span class="sidebar-text">{{ t.server_tab.nav_remote }}</span>
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
        <!-- ============ 固定域名视图 ============ -->
        <div v-if="serverMode === 'local'" class="server-sub-view">
          <!-- ============ 临时链接（临时域名） ============ -->
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
                    {{ qt.status === 'running' ? '在线' : '生成中...' }}
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

          <!-- ============ 绑定域名（命名隧道） ============ -->
          <div v-if="localSubMode === 'named'" class="server-sub-view">
          <!-- 固定域名列表卡片：创建 / 刷新 / 运行中(N) 收进右上角，
               每行的启停、修改、删除与密码锁都做进行内操作 -->
          <div class="fluent-card table-card">
            <div class="client-title-row">
              <h3 class="card-title client-title">{{ t.server_tab.local_list_title }}</h3>
              <div class="client-title-actions">
                <button class="fluent-btn small primary" @click="openNamedCreateModal">
                  <span class="btn-icon">＋</span>
                  {{ t.server_tab.btn_create }}
                </button>
                <button
                  class="fluent-btn small"
                  :disabled="refreshingTunnels.local"
                  @click="handleRefreshTunnels('local')"
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
                    <th class="col-type">{{ t.server_tab.headers.type }}</th>
                    <th class="col-created">{{ t.server_tab.headers.created }}</th>
                    <th class="col-hostname">{{ t.server_tab.headers.hostname }}</th>
                    <th class="col-connections">{{ t.server_tab.headers.connections }}</th>
                    <th class="col-status">{{ t.server_tab.headers.status }}</th>
                    <th class="col-actions">{{ t.server_tab.headers.actions }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="tunnel in localTunnelList"
                    :key="tunnel.id"
                    :class="{ selected: selectedTunnel?.id === tunnel.id }"
                    @click="selectTunnel(tunnel)"
                  >
                    <td class="col-id mono" :title="tunnel.id">{{ tunnel.id }}</td>
                    <td class="col-name font-bold">{{ tunnel.name }}</td>
                    <td class="col-type">
                      <span class="type-badge type-local">{{ t.server_tab.type_local }}</span>
                    </td>
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
                        @click.stop="openNamedEditModal(tunnel)"
                      >✎</button>
                      <button
                        class="row-action-btn danger"
                        :title="t.server_tab.btn_delete"
                        @click.stop="promptDeleteTunnel(tunnel)"
                      >🗑</button>
                    </td>
                  </tr>
                  <tr v-if="localTunnelList.length === 0">
                    <td colspan="8" class="empty-table">
                      {{ refreshingTunnels.local ? '正在刷新列表...' : '未发现隧道' }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- DNS 路由绑定卡片：标题行 + 添加按钮 + 通栏已绑定域名表 -->
          <div class="fluent-card form-card dns-route-card">
            <div class="dns-route-title-row">
              <h3 class="card-title dns-route-title">{{ t.server_tab.dns_section }}</h3>
              <button class="fluent-btn small primary" @click="openDnsAddModal">
                <span class="btn-icon">＋</span>
                {{ t.server_tab.dns_add_btn }}
              </button>
            </div>

            <!-- 已绑定域名管理：只列出固定域名的绑定记录，每条可直接改名 / 解绑 -->
            <div class="dns-bound-block">
              <div class="fluent-table-wrapper">
                <table class="fluent-table dns-bound-table">
                  <thead>
                    <tr>
                      <th class="col-tunnel-name">{{ t.server_tab.dns_col_tunnel }}</th>
                      <th class="col-bound-hostname">{{ t.server_tab.headers.hostname }}</th>
                      <th class="col-bound-lock">{{ t.server_tab.headers.lock }}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <!-- 按隧道聚合：一个隧道占一行，该隧道下的多个域名以标签换行排列。
                         改 / 解绑 / 密码锁都内嵌在各自的域名 chip 里，每个域名独立上锁。 -->
                    <tr v-for="group in dnsBoundGroups" :key="group.tunnelId">
                      <td class="col-tunnel-name font-bold" :title="group.tunnelName">{{ group.tunnelName }}</td>
                      <td class="col-bound-hostname">
                        <div class="hostname-chip-list">
                          <span
                            v-for="rec in group.records"
                            :key="rec.recordId"
                            class="hostname-chip"
                            :class="{ locked: lockOf(rec.hostname) }"
                          >
                            <span
                              class="hostname-chip-text"
                              :title="rec.hostname + ' · ' + t.server_tab.click_to_copy"
                              @click="copyHostname(rec.hostname)"
                            >{{ rec.hostname }}</span>
                            <span
                              v-if="lockOf(rec.hostname)"
                              class="chip-lock-tag"
                            >🔒 {{ t.server_tab.lock_on }}</span>
                            <span
                              v-else
                              class="chip-lock-tag off"
                            >{{ t.server_tab.lock_off }}</span>
                            <button
                              class="chip-action-btn"
                              :title="t.server_tab.dns_edit_title"
                              @click.stop="promptEditDnsRoute(rec)"
                            >✎</button>
                            <button
                              class="chip-action-btn danger"
                              :title="t.server_tab.btn_unbind"
                              @click.stop="promptUnbindDnsRoute(rec)"
                            >🗑</button>
                          </span>
                        </div>
                      </td>
                      <td class="col-bound-lock">
                        <div class="hostname-chip-list">
                          <div
                            v-for="rec in group.records"
                            :key="'lock-' + rec.recordId"
                            class="lock-chip-row"
                          >
                            <template v-if="lockOf(rec.hostname)">
                              <span
                                class="lock-cred-line mono"
                                :title="t.server_tab.lock_account_label + ' · ' + t.server_tab.click_to_copy"
                                @click.stop="copyText(lockOf(rec.hostname)?.clientId || '')"
                              >{{ lockOf(rec.hostname)?.clientId }}</span>
                              <span
                                class="lock-cred-line mono"
                                :title="t.server_tab.lock_secret_label + ' · ' + t.server_tab.click_to_copy"
                                @click.stop="copyText(lockOf(rec.hostname)?.clientSecret || '')"
                              >{{ lockOf(rec.hostname)?.clientSecret }}</span>
                              <button
                                class="chip-action-btn"
                                :title="t.server_tab.btn_rotate_password"
                                :disabled="isLockMutating"
                                @click.stop="promptRotatePassword(rec.hostname)"
                              >🔁</button>
                              <button
                                class="chip-action-btn danger"
                                :title="t.server_tab.btn_unlock"
                                :disabled="isLockMutating"
                                @click.stop="promptUnlockHostname(rec.hostname)"
                              >🔓</button>
                            </template>
                            <button
                              v-else
                              class="chip-action-btn primary"
                              :title="t.server_tab.btn_lock"
                              :disabled="isLockMutating"
                              @click.stop="promptLockHostname(rec.hostname)"
                            >🔒 {{ t.server_tab.btn_lock }}</button>
                          </div>
                        </div>
                      </td>
                    </tr>
                    <tr v-if="dnsBoundGroups.length === 0">
                      <td colspan="3" class="empty-table">{{ t.server_tab.dns_bound_empty }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>
          </div>
        </div>

        <!-- ============ 云端托管视图 ============ -->
        <div v-if="serverMode === 'remote'" class="server-sub-view">
          <!-- 云端托管列表卡片：一条隧道一行，行内 ▶/⏹ 直接启停。
               启动用的 Token 由后端拿 cert.pem 现取，界面上不出现 Token，也不落盘。 -->
          <div class="fluent-card table-card">
            <div class="client-title-row">
              <h3 class="card-title client-title">{{ t.server_tab.remote_list_title }}</h3>
              <div class="client-title-actions">
                <button
                  class="fluent-btn small"
                  :disabled="refreshingTunnels.remote"
                  @click="handleRefreshTunnels('remote')"
                >
                  <span class="btn-icon" :class="{ spinning: refreshingTunnels.remote }">🔄</span>
                  {{ refreshingTunnels.remote ? t.server_tab.btn_refreshing : t.server_tab.btn_refresh }}
                </button>
                <!-- 删除按钮就放在刷新旁边，作用于列表里选中的那条隧道，点开先弹确认框 -->
                <button
                  class="fluent-btn small danger-outline"
                  @click="promptDeleteRemoteTunnel()"
                  :disabled="!selectedRemoteTunnel"
                >
                  <span class="btn-icon">🗑️</span>
                  {{ t.server_tab.btn_delete }}
                </button>
                <div class="status-pill" :class="remoteRunningCount > 0 ? 'online' : 'offline'">
                  {{ remoteRunningCount > 0
                    ? `${t.server_tab.status_remote_running} (${remoteRunningCount})`
                    : t.server_tab.status_remote_stopped }}
                </div>
              </div>
            </div>

            <div class="fluent-table-wrapper">
              <table class="fluent-table">
                <thead>
                  <tr>
                    <th class="col-id">{{ t.server_tab.headers.id }}</th>
                    <th class="col-name">{{ t.server_tab.headers.name }}</th>
                    <th class="col-type">{{ t.server_tab.headers.type }}</th>
                    <th class="col-created">{{ t.server_tab.headers.created }}</th>
                    <th class="col-hostname">{{ t.server_tab.headers.hostname }}</th>
                    <th class="col-connections">{{ t.server_tab.headers.connections }}</th>
                    <th class="col-status">{{ t.server_tab.headers.status }}</th>
                    <th class="col-actions">{{ t.server_tab.headers.actions }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="tunnel in remoteTunnelList"
                    :key="tunnel.id"
                    :class="{ selected: selectedRemoteTunnel?.id === tunnel.id }"
                    @click="selectedRemoteTunnel = tunnel"
                  >
                    <td class="col-id mono" :title="tunnel.id">{{ tunnel.id }}</td>
                    <td class="col-name font-bold">{{ tunnel.name }}</td>
                    <td class="col-type">
                      <span class="type-badge type-remote">{{ t.server_tab.type_remote }}</span>
                    </td>
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
                      <span class="tunnel-status" :class="{ online: isRemoteRunning(tunnel.id) }">
                        <span
                          class="status-dot"
                          :class="isRemoteRunning(tunnel.id) ? 'green' : 'gray'"
                        ></span>
                        {{ isRemoteRunning(tunnel.id) ? t.server_tab.status_remote_running : t.server_tab.status_remote_stopped }}
                      </span>
                    </td>
                    <td class="col-actions">
                      <button
                        class="row-action-btn"
                        :class="isRemoteRunning(tunnel.id) ? 'danger' : 'primary'"
                        :title="isRemoteRunning(tunnel.id) ? t.server_tab.btn_stop_remote : t.server_tab.btn_start_remote"
                        @click.stop="isRemoteRunning(tunnel.id) ? handleStopRemoteTunnel(tunnel) : handleStartRemoteTunnel(tunnel)"
                      >
                        <span v-if="isRemoteRunning(tunnel.id)" class="icon-square"></span>
                        <span v-else class="icon-triangle"></span>
                      </button>
                    </td>
                  </tr>
                  <tr v-if="remoteTunnelList.length === 0">
                    <td colspan="8" class="empty-table">
                      {{ refreshingTunnels.remote ? '正在刷新列表...' : '未发现隧道' }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- 云端配置卡片：按隧道名称分组，每条隧道下三块 —— 已发布应用程序路由 / 主机名路由 / CIDR 路由。
               这三块是 Cloudflare 面板上三个独立页面的数据、三个不同接口，互不相关，所以分开列。 -->
          <div class="fluent-card form-card remote-config-card">
            <h3 class="card-title">{{ t.server_tab.remote_config_title }}</h3>
            <div class="remote-config-body">
              <template v-if="remoteConfigGroups.length">
                <div
                  v-for="g in remoteConfigGroups"
                  :key="g.key"
                  class="remote-config-group"
                >
                  <!-- 这里只放隧道名（ID 在 tooltip 里）：API 的 source 与 version 都是噪音
                       （一个语义是「配置托管方」，放进「云端配置」卡片里自相矛盾；另一个长得像软件版本号），
                       前端已不接收这两个字段。 -->
                  <div class="remote-config-group-title" :title="g.tooltip">
                    <span class="remote-config-group-name">{{ g.name }}</span>
                  </div>
                  <!-- 1. 已发布应用程序路由（= ingress，/cfd_tunnel/{id}/configurations） -->
                  <div class="remote-config-section">
                    <div class="remote-config-section-title">{{ t.server_tab.config_sec_published }}</div>
                    <pre v-if="g.published">{{ g.published }}</pre>
                    <div v-else-if="g.ingressError" class="remote-config-section-error">{{ t.server_tab.config_load_failed }}：{{ g.ingressError }}</div>
                    <div v-else class="remote-config-section-empty">{{ t.server_tab.config_none }}</div>
                  </div>
                  <!-- 2. 主机名路由（/zerotrust/routes/hostname，独立于 ingress） -->
                  <div class="remote-config-section">
                    <div class="remote-config-section-title">{{ t.server_tab.config_sec_hostname }}</div>
                    <pre v-if="g.hostname">{{ g.hostname }}</pre>
                    <div v-else-if="g.hostnameError" class="remote-config-section-error">{{ t.server_tab.config_load_failed }}：{{ g.hostnameError }}</div>
                    <div v-else class="remote-config-section-empty">{{ t.server_tab.config_none }}</div>
                  </div>
                  <!-- 3. CIDR 路由（/teamnet/routes） -->
                  <div class="remote-config-section">
                    <div class="remote-config-section-title">{{ t.server_tab.config_sec_cidr }}</div>
                    <pre v-if="g.cidr">{{ g.cidr }}</pre>
                    <div v-else-if="g.cidrError" class="remote-config-section-error">{{ t.server_tab.config_load_failed }}：{{ g.cidrError }}</div>
                    <div v-else class="remote-config-section-empty">{{ t.server_tab.config_none }}</div>
                  </div>
                </div>
              </template>
              <div v-else class="remote-config-empty">{{ t.server_tab.remote_config_empty }}</div>
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
                <span class="tile-title">{{ t.misc_tab.btn_install || '安装 cloudflared' }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_install_desc || '自动检测操作系统与CPU架构并下载至应用目录' }}</span>
              </div>
            </button>

            <!-- 2. 打开本地配置文件目录 -->
            <button class="tile-btn" @click="handleOpenConfigDir">
              <span class="tile-icon">📂</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_open_config_dir || '打开本地配置文件目录' }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_open_config_dir_desc || '在文件资源管理器中查看凭证目录（%USERPROFILE%\\.cloudflared）' }}</span>
              </div>
            </button>

            <!-- 3. 授权登录按钮 -->
            <button class="tile-btn" @click="handleCloudflaredLogin">
              <span class="tile-icon">🔑</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_login }}</span>
                <span class="tile-desc">打开浏览器进行 Cloudflare 账户授权</span>
              </div>
            </button>

            <!-- 4. 检查版本按钮 -->
            <button class="tile-btn" @click="handleCheckVersion">
              <span class="tile-icon">ℹ️</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_check_version }}</span>
                <span class="tile-desc">查看项目目录下当前生效的 cloudflared 版本</span>
              </div>
            </button>

            <!-- 5. 在线更新按钮 -->
            <button class="tile-btn" @click="handleUpdateCloudflared">
              <span class="tile-icon">⚡</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_update }}</span>
                <span class="tile-desc">检测项目目录下的 cloudflared 并执行升级</span>
              </div>
            </button>

            <!-- 6. 前往官网下载 -->
            <button class="tile-btn" @click="handleDownloadCloudflared">
              <span class="tile-icon">⬇️</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_download }}</span>
                <span class="tile-desc">前往官方 GitHub 仓库查看最新 Releases</span>
              </div>
            </button>
          </div>

          <!-- 醒目的敏感凭证安全防泄露警告提示 (正下方红色醒目提示) -->
          <div class="config-warning-banner">
            <span class="warning-icon">⚠️</span>
            <span class="warning-text">{{ t.misc_tab.config_dir_warning || '提示：请不要将 cert.pem 证书文件和 .json 隧道配置文件展示或分享给任何人，以免造成隐私泄露和隧道被盗用！' }}</span>
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
        title="按住上下拖拽调整控制台高度"
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

    <!-- Win11 确认删除隧道模态弹窗（固定域名隧道 / 云端托管隧道共用） -->
    <div v-if="showDeleteModal" class="fluent-modal-overlay" @click.self="cancelDelete">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">
            ⚠️ {{ pendingDelete?.scope === 'remote'
              ? t.server_tab.remote_delete_confirm_title
              : t.server_tab.errors.delete_confirm_title }}
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
                v-if="localTunnelList.length > 0"
                v-model="dnsRoute.name"
                class="fluent-input fluent-select"
              >
                <option
                  v-for="tn in localTunnelList"
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

    <!-- 固定域名：创建隧道弹窗（创建 + 可选绑定域名 + 可选上锁，一步到位） -->
    <div v-if="showNamedCreateModal" class="fluent-modal-overlay">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">🔗 {{ t.server_tab.named_create_title }}</h3>
        </div>
        <div class="modal-body">
          <div class="fluent-form-group">
            <label class="form-label">
              {{ t.server_tab.tunnel_name }}
              <span class="required">*</span>
            </label>
            <div class="input-container">
              <input
                type="text"
                v-model="serverConfig.name"
                :placeholder="t.server_tab.tunnel_name_placeholder"
                :class="['fluent-input', { 'input-error': serverNameHasError }]"
                @input="onServerNameInput"
                @keydown.enter="confirmNamedCreate"
              />
            </div>
            <div v-if="serverNameHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.server_tab.errors.tunnel_invalid }}
            </div>
          </div>

          <div class="fluent-form-group">
            <label class="form-label">{{ t.server_tab.protocol_label }}</label>
            <div class="input-container">
              <select v-model="serverConfig.protocol" class="fluent-input fluent-select">
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

          <div v-if="serverAddressMode !== 'none'" class="fluent-form-group">
            <label class="form-label">
              {{ serverAddressMode === 'socket' ? t.server_tab.unix_socket_label : t.server_tab.port }}
              <span class="required">*</span>
            </label>
            <div class="input-container">
              <input
                v-if="serverAddressMode === 'port'"
                type="text"
                v-model="serverConfig.port"
                :placeholder="t.server_tab.port_placeholder"
                :class="['fluent-input', { 'input-error': serverPortHasError }]"
                @input="onServerPortInput"
                @keydown.enter="confirmNamedCreate"
              />
              <input
                v-else
                type="text"
                v-model="serverConfig.unixSocket"
                :placeholder="t.server_tab.unix_socket_placeholder"
                class="fluent-input"
                @input="onServerUnixSocketInput"
                @keydown.enter="confirmNamedCreate"
              />
            </div>
            <div v-if="serverAddressMode === 'port' && serverPortHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.server_tab.errors.port_invalid }}
            </div>
          </div>

          <!-- 可选：创建后顺手绑定域名（cloudflared tunnel route dns） -->
          <div class="fluent-form-group">
            <label class="form-label">{{ t.server_tab.named_create_domain_label }}</label>
            <div class="input-container">
              <input
                type="text"
                v-model="namedCreateDomain"
                :placeholder="t.server_tab.named_create_domain_placeholder"
                :class="['fluent-input', { 'input-error': namedCreateDomainHasError }]"
                @input="onNamedCreateDomainInput"
                @keydown.enter="confirmNamedCreate"
              />
            </div>
            <div v-if="namedCreateDomainHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.server_tab.errors.dns_domain_invalid }}
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="showNamedCreateModal = false">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn primary" @click="confirmNamedCreate" :disabled="isCreatingTunnel || isLockMutating">
            {{ t.server_tab.btn_create }}
          </button>
        </div>
      </div>
    </div>

    <!-- 固定域名：修改隧道弹窗（协议 / 端口 + 密码锁，保存后运行中的隧道自动重启生效） -->
    <div v-if="showNamedEditModal" class="fluent-modal-overlay">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">✎ {{ t.server_tab.named_edit_title }}</h3>
        </div>
        <div class="modal-body">
          <p v-if="namedEditTarget" class="modal-context font-bold">{{ namedEditTarget.name }}</p>

          <div class="fluent-form-group">
            <label class="form-label">{{ t.server_tab.protocol_label }}</label>
            <div class="input-container">
              <select v-model="namedEdit.protocol" class="fluent-input fluent-select">
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

          <div v-if="namedEditAddressMode !== 'none'" class="fluent-form-group">
            <label class="form-label">
              {{ namedEditAddressMode === 'socket' ? t.server_tab.unix_socket_label : t.server_tab.port }}
              <span class="required">*</span>
            </label>
            <div class="input-container">
              <input
                v-if="namedEditAddressMode === 'port'"
                type="text"
                v-model="namedEdit.port"
                :placeholder="t.server_tab.port_placeholder"
                :class="['fluent-input', { 'input-error': namedEditPortHasError }]"
                @input="namedEditPortHasError = namedEdit.port.length > 0 && !isPortValid(namedEdit.port)"
              />
              <input
                v-else
                type="text"
                v-model="namedEdit.unixSocket"
                :placeholder="t.server_tab.unix_socket_placeholder"
                class="fluent-input"
              />
            </div>
            <div v-if="namedEditAddressMode === 'port' && namedEditPortHasError" class="error-tip">
              <span class="error-icon">⚠️</span>
              {{ t.server_tab.errors.port_invalid }}
            </div>
          </div>

          <div v-if="namedEditTarget && isTunnelRunning(namedEditTarget.name)" class="modal-hint warn">
            {{ t.server_tab.edit_restart_hint }}
          </div>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="showNamedEditModal = false">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn primary" @click="confirmNamedEdit" :disabled="isCreatingTunnel || isLockMutating">
            {{ t.server_tab.btn_save }}
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
import { LANG_ORDER, LANG_DATA } from './i18n';
import { LangKey, TunnelInfo, QuickTunnelItem, ClientTunnelItem, LogEntry, DnsBinding } from './types';
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
    showToast('UI 提示音效已开启');
  } else {
    showToast('UI 提示音效已静音');
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

// 语言状态与自定义下拉菜单
const savedLang = (localStorage.getItem('app_lang') as LangKey) || 'zh_CN';
const currentLang = ref<LangKey>(LANG_DATA[savedLang] ? savedLang : 'zh_CN');
const isLangDropdownOpen = ref(false);
const langDropdownRef = ref<HTMLDivElement | null>(null);

const currentLangItem = computed(() => {
  return LANG_ORDER.find(item => item.key === currentLang.value) || LANG_ORDER[0];
});

const t = computed(() => LANG_DATA[currentLang.value] || LANG_DATA['zh_CN']);

const selectLanguage = (key: LangKey) => {
  currentLang.value = key;
  isLangDropdownOpen.value = false;
  localStorage.setItem('app_lang', key);
  // 切换语言需要重载页面（让 Vue 重新挂载整个 DOM，彻底防止错乱）。
  // 重载会丢失内存里的视图状态，所以先把当前所在界面暂存到 sessionStorage，
  // 由 setup 阶段同步恢复，避免重载后跳回「配置」页。
  try {
    sessionStorage.setItem('restore_view_on_reload', JSON.stringify({
      tab: currentTab.value,
      sidebarCollapsed: sidebarCollapsed.value,
      sidebarOpen: sidebarOpen.value,
    }));
  } catch {}
  window.location.reload();
};

// 选项卡状态（点击软件默认进入「配置」页）
const currentTab = ref('misc');

// 侧边栏状态：整体默认折叠（图标栏），服务端下边栏默认折叠
const sidebarCollapsed = ref(true);
const sidebarOpen = ref<Record<string, boolean>>({
  server: false,
  client: false,
  misc: false,
});

// 若上一动作是「切换语言」触发的页面重载，则同步还原当时的视图位置，
// 避免重载后跳回「配置」页（server_mode / local_sub_mode 本就持久化，无需再还原）。
try {
  const raw = sessionStorage.getItem('restore_view_on_reload');
  if (raw) {
    sessionStorage.removeItem('restore_view_on_reload');
    const saved = JSON.parse(raw) as {
      tab?: string;
      sidebarCollapsed?: boolean;
      sidebarOpen?: Record<string, boolean>;
    };
    if (typeof saved.tab === 'string') currentTab.value = saved.tab;
    if (typeof saved.sidebarCollapsed === 'boolean') sidebarCollapsed.value = saved.sidebarCollapsed;
    if (saved.sidebarOpen && typeof saved.sidebarOpen === 'object') {
      sidebarOpen.value = { ...sidebarOpen.value, ...saved.sidebarOpen };
    }
  }
} catch {}

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

// 服务端下边栏的三视图切换：临时链接 / 固定域名 / 云端托管
const switchServerView = (view: 'quick' | 'named' | 'remote') => {
  if (view === 'remote') {
    switchServerMode('remote');
  } else {
    switchServerMode('local');
    switchLocalSubMode(view);
  }
  sidebarOpen.value.server = true;
  switchTab('server');
};

// 当前是否处于服务端某个子视图
const isServerView = (view: 'quick' | 'named' | 'remote') => {
  if (view === 'remote') return serverMode.value === 'remote';
  return serverMode.value === 'local' && localSubMode.value === view;
};

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

// 输入错误校验状态
const serverNameHasError = ref(false);
const serverPortHasError = ref(false);
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

// 判断服务端表单是否满足启动条件（hello_world 无需端口，unix 协议需要套接字路径）
// 注：顶部常驻表单已移除，创建/修改都走弹窗，这里保留给弹窗复用的校验逻辑见 confirmNamedCreate

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
// 刷新态按列表分开：固定域名 / 云端托管各一份。
// 早先两处共用一个布尔值，于是「固定域名」正在刷新时，「云端托管」的刷新按钮
// 也会一起变灰 —— 而且这个刷新会去等 Cloudflare API（网络不通时长达二三十秒），
// 表现出来就是「明明没在刷这个列表，按钮却点不动」。
const refreshingTunnels = ref<{ local: boolean; remote: boolean; quick: boolean }>({ local: false, remote: false, quick: false });
const isCreatingTunnel = ref(false);
const isDownloadingCloudflared = ref(false);

// 服务端模式：本地 / 远程 切换
const serverMode = ref<'local' | 'remote'>(localStorage.getItem('server_mode') === 'remote' ? 'remote' : 'local');

const switchServerMode = (mode: 'local' | 'remote') => {
  serverMode.value = mode;
  localStorage.setItem('server_mode', mode);
  // 切到云端托管时取云端配置：走 API 读取，隧道没跑起来也能看到规则。
  // 这里**不**强刷 —— 缓存没过期就直接用，否则每次切视图都会重拉全部隧道，白白卡一下。
  if (mode === 'remote') void refreshRemoteConfigs();
};

// 固定域名二级模式：临时链接(临时域名) / 命名隧道(绑定域名)
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
//   none   —— hello_world 内置测试服务器，不需要填写任何地址
type AddressMode = 'port' | 'socket' | 'none';
const addressModeOf = (protocol: string): AddressMode => {
  if (protocol === 'hello_world') return 'none';
  if (protocol === 'unix' || protocol === 'unix+tls') return 'socket';
  return 'port';
};

const quickAddressMode = computed(() => addressModeOf(quickConfig.value.protocol));
const serverAddressMode = computed(() => addressModeOf(serverConfig.value.protocol));

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
  if (qt.protocol === 'hello_world') return 'hello_world 内置测试服务器';
  if (qt.protocol === 'unix' || qt.protocol === 'unix+tls') return `${qt.protocol}:${qt.port}`;
  return `${qt.protocol}://127.0.0.1:${qt.port}`;
};

// 云端托管不再需要用户填 Token：启动时由 Rust 侧拿 cert.pem 现取，
// 进程表 key 就是隧道 ID，前端靠它跟隧道列表里的行对上号，Token 全程不落盘。
const remoteRunningKeys = ref<string[]>([]);
const isRemoteRunning = (tunnelId: string) => remoteRunningKeys.value.includes(tunnelId);
const remoteRunningCount = computed(() => remoteRunningKeys.value.length);

// 云端配置（只读）。面板上这些数据其实分三块，来源是三个互不相关的接口：
//   1. 已发布应用程序路由 → GET .../cfd_tunnel/{id}/configurations 的 ingress
//   2. 主机名路由        → GET .../zerotrust/routes/hostname
//   3. CIDR 路由         → GET .../teamnet/routes
// 数据都来自 Cloudflare API 而不是 cloudflared 的运行日志 —— 隧道没跑起来也能看到配置。
type TunnelIngressRule = { hostname: string; path: string; service: string };
type TunnelHostnameRoute = { hostname: string; comment: string };
type TunnelCidrRoute = { network: string; comment: string };
type TunnelRouteSet = {
  hostname_routes: TunnelHostnameRoute[];
  cidr_routes: TunnelCidrRoute[];
  hostname_error: string | null;
  cidr_error: string | null;
};
type TunnelCloudInfo = {
  rules: TunnelIngressRule[];
  hostnameRoutes: TunnelHostnameRoute[];
  cidrRoutes: TunnelCidrRoute[];
  // 三块各自的读取失败原因，空串表示读到了（列表本身可能为空）
  ingressError: string;
  hostnameError: string;
  cidrError: string;
};
const remoteConfigs = ref<Record<string, TunnelCloudInfo>>({});

// invoke 被拒绝时抛出来的既可能是字符串（Rust 侧 Result<_, String>），也可能是别的对象，
// 统一转成能直接显示的一行文本。
const errorText = (e: unknown): string =>
  typeof e === 'string' ? e : String((e as Error)?.message ?? e);

// 拉一条隧道的云端配置：已发布应用程序路由 + 主机名路由 + CIDR 路由。
// 三块分别容错：某一块失败只在那块里显示「读取失败」，另外两块照常显示 ——
// 否则主机名路由缺权限（403）会把已经拿到的已发布应用程序路由一起遮掉。
const fetchOneTunnelConfig = async (
  tn: TunnelInfo
): Promise<readonly [string, TunnelCloudInfo]> => {
  const [cfgRes, routeRes] = await Promise.allSettled([
    invoke<{ rules: TunnelIngressRule[] }>('fetch_tunnel_config', {
      tunnelId: tn.id,
    }),
    invoke<TunnelRouteSet>('fetch_tunnel_routes', { tunnelId: tn.id }),
  ]);
  const info: TunnelCloudInfo = {
    rules: cfgRes.status === 'fulfilled' ? cfgRes.value.rules : [],
    ingressError: cfgRes.status === 'rejected' ? errorText(cfgRes.reason) : '',
    // 演示模式（浏览器里跑）可能返回 null，这里兜一层，别让整条刷新链炸掉
    hostnameRoutes: routeRes.status === 'fulfilled' ? routeRes.value?.hostname_routes ?? [] : [],
    cidrRoutes: routeRes.status === 'fulfilled' ? routeRes.value?.cidr_routes ?? [] : [],
    hostnameError:
      routeRes.status === 'fulfilled'
        ? routeRes.value?.hostname_error || ''
        : errorText(routeRes.reason),
    cidrError:
      routeRes.status === 'fulfilled'
        ? routeRes.value?.cidr_error || ''
        : errorText(routeRes.reason),
  };
  return [tn.id, info] as const;
};

// 云端配置的缓存：**切视图不该打网络请求**。
//
// 每拉一轮要对每条隧道发 2 个 IPC，每个 IPC 背后是一次阻塞式 HTTPS，实测单次约 340ms 且
// 不复用连接 —— 6 条隧道就是 12 个请求。原先切到「云端托管」必然重拉一遍，
// 于是「从云端托管切到固定域名 / 临时链接」永远卡在等待里（隧道全删光就不卡，正是这个原因）。
//
// 三条规矩：
//   * TTL 内且隧道集合没变 → 直接用缓存，0 次 IPC，切视图瞬时完成
//   * 已有在途请求 → 等它落地，连点也不会堆出一串
//   * force（手动点刷新 / 启停隧道）才强制重拉
const REMOTE_CONFIG_TTL_MS = 60_000;
// 一轮并发几条隧道：每条要发 2 个 HTTPS（每次还都新建连接），一次全铺开容易被限流
const REMOTE_CONFIG_CONCURRENCY = 3;
let remoteConfigsFetchedAt = 0;
let remoteConfigsSignature = '';
let remoteConfigsInFlight: Promise<void> | null = null;

// 隧道集合的指纹：增删隧道后配置会变，缓存必须跟着失效
const tunnelSetSignature = (items: TunnelInfo[]) => items.map(t => t.id).sort().join(',');

// 让缓存立即失效，下次读取时重拉（隧道增删、启停之后调用）
const invalidateRemoteConfigs = () => {
  remoteConfigsFetchedAt = 0;
};

const refreshRemoteConfigs = async (opts: { force?: boolean } = {}): Promise<void> => {
  const items = remoteTunnelList.value;
  if (!items.length) {
    remoteConfigs.value = {};
    remoteConfigsFetchedAt = 0;
    remoteConfigsSignature = '';
    return;
  }
  const signature = tunnelSetSignature(items);
  const cacheValid =
    !opts.force &&
    remoteConfigsSignature === signature &&
    remoteConfigsFetchedAt > 0 &&
    Date.now() - remoteConfigsFetchedAt < REMOTE_CONFIG_TTL_MS;
  if (cacheValid) return;
  // 已经有一轮在路上：先等它，别叠加请求
  if (remoteConfigsInFlight) {
    await remoteConfigsInFlight;
    // 强刷时那一轮可能基于旧的隧道集合，落地后按最新签名再来一次
    if (opts.force) await refreshRemoteConfigs(opts);
    return;
  }

  // 用局部 job 承接，别直接 return 那个可变的模块级引用：
  // 它在 finally 里会被置回 null，TS 无法窄化 `Promise<void> | null`
  const job: Promise<void> = (async () => {
    const next: Record<string, TunnelCloudInfo> = {};
    // 分批串行：每批 REMOTE_CONFIG_CONCURRENCY 条隧道并发，避免一次铺开 2N 个请求
    for (let i = 0; i < items.length; i += REMOTE_CONFIG_CONCURRENCY) {
      const batch = items.slice(i, i + REMOTE_CONFIG_CONCURRENCY);
      const done = await Promise.all(batch.map(fetchOneTunnelConfig));
      for (const [id, info] of done) next[id] = info;
    }
    remoteConfigs.value = next;
    remoteConfigsFetchedAt = Date.now();
    remoteConfigsSignature = signature;
  })().finally(() => {
    remoteConfigsInFlight = null;
  });
  remoteConfigsInFlight = job;
  return job;
};

// 规则渲染成「匹配目标 → 源站」的文本行。hostname 为空即 ingress 末尾的兜底规则，
// 显示成「(默认)」；带 path 的规则把 path 缀在域名后面一起显示。
const formatIngressRules = (cfg?: TunnelCloudInfo) => {
  if (!cfg || !cfg.rules.length) return '';
  return cfg.rules
    .map(r => {
      const target = r.path ? `${r.hostname || '(默认)'}${r.path}` : r.hostname || '(默认)';
      return `${target}  →  ${r.service}`;
    })
    .join('\n');
};

// 主机名路由 / CIDR 路由：一行一条，有「描述」就缀在后面。
const formatRouteLines = (rows: { value: string; comment: string }[]) =>
  rows
    .map(r => {
      const value = r.value.trim();
      if (!value) return '';
      return r.comment.trim() ? `${value}  ·  ${r.comment.trim()}` : value;
    })
    .filter(Boolean)
    .join('\n');

// 配置卡片按隧道分组：标题用隧道名称，隧道 ID 只放进 tooltip，
// 否则一列 8e1b8616 / 511e2469 根本分不清是哪条隧道。
type RemoteConfigGroup = {
  key: string;
  name: string;
  tooltip: string;
  published: string;
  hostname: string;
  cidr: string;
  ingressError: string;
  hostnameError: string;
  cidrError: string;
};

const toConfigGroup = (key: string, name: string, cfg?: TunnelCloudInfo): RemoteConfigGroup => {
  const info: TunnelCloudInfo = cfg ?? {
    rules: [],
    hostnameRoutes: [],
    cidrRoutes: [],
    ingressError: '',
    hostnameError: '',
    cidrError: '',
  };
  return {
    key,
    name,
    tooltip: `隧道 ID: ${key}`,
    published: formatIngressRules(cfg),
    hostname: formatRouteLines(
      info.hostnameRoutes.map(r => ({ value: r.hostname, comment: r.comment })),
    ),
    cidr: formatRouteLines(info.cidrRoutes.map(r => ({ value: r.network, comment: r.comment }))),
    ingressError: info.ingressError,
    hostnameError: info.hostnameError,
    cidrError: info.cidrError,
  };
};

// 列表里的每条隧道都出组，哪怕三块全空 —— 否则「这条隧道没有主机名路由」
// 跟「压根没拉到」看起来一模一样，分不清。
const remoteConfigGroups = computed(() => {
  const groups = remoteTunnelList.value.map(tn =>
    toConfigGroup(tn.id, tn.name || tn.id, remoteConfigs.value[tn.id]),
  );
  // 后端在跑但隧道列表里还没有的（刚启动就刷新失败等）：按 ID 兜底显示
  const known = new Set(groups.map(g => g.key));
  for (const key of remoteRunningKeys.value) {
    if (known.has(key)) continue;
    groups.push(toConfigGroup(key, key, remoteConfigs.value[key]));
  }
  return groups;
});

// 隧道列表与选中项
const tunnelList = ref<TunnelInfo[]>([]);
const selectedTunnel = ref<TunnelInfo | null>(null);

const localTunnelList = computed(() => tunnelList.value.filter(t => t.tunnel_type === 'local'));
const remoteTunnelList = computed(() => tunnelList.value.filter(t => t.tunnel_type === 'remote'));

// 云端托管列表里被选中的那一行：顶部「删除」按钮作用于它（每行另有独立的 🗑 入口）
const selectedRemoteTunnel = ref<TunnelInfo | null>(null);

// 服务端三类隧道状态胶囊的「运行中 (数量)」统计
const localRunningCount = computed(() =>
  localTunnelList.value.filter(x => isTunnelRunning(x.name)).length,
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

// 上锁 / 换密码成功后的凭据展示弹窗
const showLockInfoModal = ref(false);
const lockInfoDraft = ref<{ hostname: string; clientId: string; clientSecret: string } | null>(null);

// 配置页的 Access Token：留空 = 用「授权登录」凭证。
// 该凭证需要 Access: Apps and Policies 与 Access: Service Tokens 两个编辑权限（见 README）。
const accessTokenInput = ref(localStorage.getItem('access_api_token') || '');
const saveAccessToken = () => {
  accessTokenInput.value = accessTokenInput.value.trim();
  localStorage.setItem('access_api_token', accessTokenInput.value);
  appendLog('[INFO] 访问密码锁凭证（Access Token）已保存', 'info', 'misc');
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
    appendLog(`[SUCCESS] 已为域名 [${res.hostname}] 开启密码锁，只有携带访问密码的客户端能连接`, 'success', 'server');
    return true;
  } catch (err: any) {
    appendLog(`[ERROR] 上锁失败: ${errorText(err)}`, 'error', 'server');
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
    appendLog(`[ERROR] 解锁失败: ${errorText(err)}`, 'error', 'server');
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
    appendLog(`[SUCCESS] 已为域名 [${res.hostname}] 更换访问密码，旧密码作废`, 'success', 'server');
    return true;
  } catch (err: any) {
    appendLog(`[ERROR] 更换密码失败: ${errorText(err)}`, 'error', 'server');
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
  localTunnelList.value
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
    appendLog(`[WARN] 核对残留域名失败，将直接尝试补删: ${errorText(err)}`, 'warn', 'server');
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

// 待删除的隧道。固定域名隧道（local）与云端托管隧道（remote）共用同一个确认弹窗，
// scope 决定删除后刷新哪张列表、以及要不要先停掉本机正在跑的那个进程。
const pendingDelete = ref<{ id: string; name: string; scope: 'local' | 'remote' } | null>(null);

// 确认弹窗里的正文：本地与云端用的是两套文案，占位符也不一样（{name} / {target}）
const deleteConfirmMessage = computed(() => {
  const name = pendingDelete.value?.name || '';
  const tpl =
    pendingDelete.value?.scope === 'remote'
      ? t.value.server_tab.remote_delete_confirm_msg
      : t.value.server_tab.errors.delete_confirm_msg;
  return String(tpl).replace('{name}', name).replace('{target}', name);
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
    message: 'CFTunnel (Windows 11 Fluent) 已启动就绪',
    level: 'info',
    source: 'system',
  },
]);

// 格式校验触发
const onServerNameInput = () => {
  const val = serverConfig.value.name;
  serverNameHasError.value = val.length > 0 && !isTunnelNameValid(val);
  localStorage.setItem('server_tunnel_name', val);
};

const onServerPortInput = () => {
  const val = serverConfig.value.port;
  serverPortHasError.value = val.length > 0 && !isPortValid(val);
  localStorage.setItem('server_port', val);
};

const onServerUnixSocketInput = () => {
  localStorage.setItem('server_unix_socket', serverConfig.value.unixSocket);
};

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
    appendLog('复制日志失败，请检查剪贴板权限', 'error');
  }
};

// 打开外部链接
const openUrl = async (url: string) => {
  try {
    await invoke('open_external_url', { url });
    appendLog(`[INFO] 已在默认浏览器中打开: ${url}`, 'info', 'misc');
  } catch (err) {
    appendLog(`打开链接失败: ${err}`, 'error', 'misc');
  }
};

// 打开 cloudflared 的默认凭证目录（%USERPROFILE%\.cloudflared）
const handleOpenConfigDir = async () => {
  try {
    const dir = await invoke<string>('open_cloudflared_config_dir');
    showToast(`已打开配置目录: ${dir}`);
  } catch (err: any) {
    appendLog(`[ERROR] 打开配置文件目录失败: ${err}`, 'error', 'misc');
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
const handleRefreshTunnels = async (scope?: 'local' | 'remote') => {
  const key: 'local' | 'remote' = scope === 'remote' ? 'remote' : 'local';
  // 防重入：同一个列表连点不做第二次；两个列表互不影响
  if (refreshingTunnels.value[key]) return;
  refreshingTunnels.value[key] = true;
  try {
    const res = await invoke<TunnelInfo[]>('list_tunnels');
    tunnelList.value = res;

    // 绑定域名走云 API，不阻塞列表上屏与按钮恢复
    void fillTunnelHostnames();

    // 与后端对账命名隧道的运行状态（多开后靠这里把已退出的进程同步掉）
    await reconcileServerRunning();

    // 云端托管列表的刷新按钮只有 handleRefreshTunnels 这一个入口，
    // 顺手把云端隧道进程也对账一次，否则「运行中」状态会一直停在旧值上。
    // 注意条件保留 `scope !== 'local'`：启动时的自动刷新（不带 scope）也要走到，
    // 否则上次停在云端 Tab 的用户重启后，配置卡片会一直空着。
    if (scope !== 'local') {
      await refreshRemoteTunnels();
      // 云端 ingress 配置要逐条隧道发两个 API 请求，网络不通时会拖很久。
      // 列表与运行状态都已更新完，这里后台跑，不继续锁着按钮；
      // 只有用户亲手点这个列表的刷新按钮才强刷，自动刷新走缓存（见 refreshRemoteConfigs）。
      void refreshRemoteConfigs({ force: scope === 'remote' });
    }

    // 按触发刷新的列表分别统计：固定域名列表 / 云端托管
    const localCount = res.filter(x => x.tunnel_type === 'local').length;
    const remoteCount = res.filter(x => x.tunnel_type === 'remote').length;
    const scopeName = scope === 'local' ? '固定域名' : scope === 'remote' ? '云端托管' : '隧道';
    const count = scope === 'local' ? localCount : scope === 'remote' ? remoteCount : res.length;
    appendLog(`[INFO] 已刷新${scopeName}列表，共获取到 ${count} 条隧道`, 'info', 'server');
  } catch (err: any) {
    const scopeName = scope === 'local' ? '固定域名' : scope === 'remote' ? '云端托管' : '隧道';
    appendLog(`[ERROR] 刷新${scopeName}列表失败: ${err}`, 'error', 'server');
  } finally {
    refreshingTunnels.value[key] = false;
  }
};

// 创建隧道 (触发 playSuccess 音效)
// 创建弹窗状态：名称/协议/端口沿用 serverConfig（同时充当「上次用的值」的记忆），
// 域名与上锁开关是弹窗自己的状态。
const showNamedCreateModal = ref(false);
const namedCreateDomain = ref('');
const namedCreateDomainHasError = ref(false);

const onNamedCreateDomainInput = () => {
  const val = namedCreateDomain.value;
  namedCreateDomainHasError.value = val.length > 0 && !isDomainValid(val);
};

const openNamedCreateModal = () => {
  soundManager.playClick();
  namedCreateDomain.value = '';
  namedCreateDomainHasError.value = false;
  serverNameHasError.value = false;
  serverPortHasError.value = false;
  showNamedCreateModal.value = true;
};

// 创建 + 可选绑定域名，一步到位（密码锁改到「DNS 路由绑定」面板按域名上锁）：
//   1. cloudflared tunnel create
//   2. （填了域名）cloudflared tunnel route dns
const confirmNamedCreate = async () => {
  const name = serverConfig.value.name.trim();
  const port = serverConfig.value.port.trim();
  const protocol = serverConfig.value.protocol;
  const unixSocket = serverConfig.value.unixSocket.trim();
  const domain = namedCreateDomain.value.trim();

  if (!name || !isTunnelNameValid(name)) {
    serverNameHasError.value = true;
    appendLog(`[ERROR] ${t.value.server_tab.errors.tunnel_invalid}`, 'error', 'server');
    return;
  }
  if (protocol !== 'hello_world' && !isPortValid(port)) {
    serverPortHasError.value = true;
    appendLog(`[ERROR] 本地端口错误`, 'error', 'server');
    return;
  }
  if ((protocol === 'unix' || protocol === 'unix+tls') && !unixSocket) {
    appendLog(`[ERROR] unix / unix+tls 协议必须填写套接字路径`, 'error', 'server');
    return;
  }
  if (domain && !isDomainValid(domain)) {
    namedCreateDomainHasError.value = true;
    appendLog(`[ERROR] ${t.value.server_tab.errors.dns_domain_invalid}`, 'error', 'server');
    return;
  }

  soundManager.playSuccess();
  isCreatingTunnel.value = true;
  try {
    const res = await invoke<string>('create_tunnel', { name });
    appendLog(`[SUCCESS] 成功创建隧道 [${name}]: ${res}`, 'success', 'server');

    // 记住这条隧道的源站配置，列表行内「启动」直接可用
    saveTunnelCfgValues(name, protocol, port, unixSocket);

    if (domain) {
      try {
        const dnsRes = await invoke<string>('route_dns_tunnel', { name, hostname: domain });
        appendLog(`[SUCCESS] ${dnsRes}`, 'success', 'server');
      } catch (err: any) {
        appendLog(`[ERROR] 绑定域名失败: ${err}`, 'error', 'server');
        showToast(`绑定域名失败: ${err}`);
      }
    }

    // 刷新列表拿到新隧道的 ID，后续域名展示要用
    await handleRefreshTunnels();

    if (domain) await refreshHostnamesOnly();

    if (!showLockInfoModal.value) showToast(`隧道 [${name}] 创建成功！`);
    showNamedCreateModal.value = false;
  } catch (err: any) {
    appendLog(`[ERROR] 创建隧道失败: ${err}`, 'error', 'server');
  } finally {
    isCreatingTunnel.value = false;
  }
};

// 源站目标描述文案（日志 / 提示用）
const describeServerTarget = (protocol: string, port: string, unixSocket: string) => {
  if (protocol === 'hello_world') return 'hello_world 内置测试服务器';
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
    appendLog(`[WARN] 隧道 [${name}] 还没有配置协议和端口，请先点击「修改」补全`, 'warn', 'server');
    showToast(t.value.server_tab.edit_need_config);
    openNamedEditModal(tunnel);
    return;
  }
  if (saved.protocol === 'unix' || saved.protocol === 'unix+tls') {
    if (!saved.unixSocket) {
      appendLog(`[ERROR] unix / unix+tls 协议必须填写套接字路径`, 'error', 'server');
      return;
    }
  } else if (saved.protocol !== 'hello_world' && !isPortValid(saved.port)) {
    appendLog(`[ERROR] 隧道 [${name}] 保存的端口不合法，请点击「修改」更正`, 'error', 'server');
    openNamedEditModal(tunnel);
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
    showToast(`隧道 [${name}] 已启动 (${describeServerTarget(saved.protocol, saved.port, saved.unixSocket)})`);
  } catch (err: any) {
    appendLog(`[ERROR] 启动服务端隧道失败: ${err}`, 'error', 'server');
  }
};

// 修改隧道弹窗：协议 / 端口（密码锁已挪到「DNS 路由绑定」面板按域名管理）。
// 保存时：落库源站配置 → 运行中的隧道用新配置自动重启。
const showNamedEditModal = ref(false);
const namedEditTarget = ref<TunnelInfo | null>(null);
const namedEdit = ref({ protocol: 'http', port: '', unixSocket: '' });
const namedEditPortHasError = ref(false);

const namedEditAddressMode = computed(() => addressModeOf(namedEdit.value.protocol));

const openNamedEditModal = (tunnel: TunnelInfo) => {
  soundManager.playClick();
  const saved = loadTunnelCfg(tunnel.name.trim());
  namedEditTarget.value = tunnel;
  namedEdit.value = {
    protocol: saved?.protocol || 'http',
    port: saved?.port || '',
    unixSocket: saved?.unixSocket || '',
  };
  namedEditPortHasError.value = false;
  showNamedEditModal.value = true;
};

const confirmNamedEdit = async () => {
  const target = namedEditTarget.value;
  if (!target) return;
  const protocol = namedEdit.value.protocol;
  const port = namedEdit.value.port.trim();
  const unixSocket = namedEdit.value.unixSocket.trim();
  const name = target.name.trim();

  if (protocol !== 'hello_world' && !isPortValid(port)) {
    namedEditPortHasError.value = true;
    appendLog(`[ERROR] 本地端口错误`, 'error', 'server');
    return;
  }
  if ((protocol === 'unix' || protocol === 'unix+tls') && !unixSocket) {
    appendLog(`[ERROR] unix / unix+tls 协议必须填写套接字路径`, 'error', 'server');
    return;
  }

  const saved = loadTunnelCfg(name);
  const cfgChanged =
    !saved ||
    saved.protocol !== protocol ||
    saved.port !== port ||
    saved.unixSocket !== unixSocket;

  // 1. 源站配置落库
  if (cfgChanged) saveTunnelCfgValues(name, protocol, port, unixSocket);

  // 2. 运行中的隧道用新配置自动重启（配置没变就不折腾）
  if (cfgChanged && isTunnelRunning(name)) {
    try {
      await invoke<string>('stop_server_tunnel', { name });
    } catch {}
    markServerStopped(name);
    try {
      await invoke<string>('start_server_tunnel', { name, port, protocol, unixSocket });
      markServerRunning(name);
      appendLog(`[INFO] 隧道 [${name}] 已用新配置重启 (${describeServerTarget(protocol, port, unixSocket)})`, 'info', 'server');
    } catch (err: any) {
      appendLog(`[ERROR] 重启隧道失败: ${err}`, 'error', 'server');
    }
  }

  soundManager.playSuccess();
  showToast(`隧道 [${name}] 已保存`);
  showNamedEditModal.value = false;
  namedEditTarget.value = null;
};

// 停止服务端隧道 (普通点击音效)
// 省略 name 时停「表单里当前这条」；列表行内按钮会传入该行隧道名，多开时逐条停
const handleStopServer = async (name?: string) => {
  const target = (typeof name === 'string' ? name : serverConfig.value.name).trim();
  if (!target) {
    appendLog(`[ERROR] 未指定要停止的隧道名`, 'error', 'server');
    return;
  }
  soundManager.playClick();
  try {
    // 停止成功的日志由 Rust 侧统一广播，这里不再重复打印
    await invoke<string>('stop_server_tunnel', { name: target });
    markServerStopped(target);
    showToast(`隧道 [${target}] 已停止`);
  } catch (err: any) {
    appendLog(`[ERROR] 停止隧道 [${target}] 失败: ${err}`, 'error', 'server');
  }
};

// 打开「添加绑定」弹窗（隧道下拉 + 域名输入）
const openDnsAddModal = () => {
  // 没有任何固定域名隧道时无处可绑：直接提示，不弹空下拉框
  if (localTunnelList.value.length === 0) {
    soundManager.playClick();
    appendLog('[ERROR] 未发现隧道，请先创建固定域名隧道再绑定域名', 'error', 'server');
    showToast(t.value.server_tab.quick_list_empty);
    return;
  }
  soundManager.playClick();
  // 预选第一个固定域名隧道（若有），域名清空
  dnsRoute.value.name = localTunnelList.value[0]?.name || '';
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
    showToast(`DNS 路由绑定成功: ${domain} → ${name}`);
    // 绑定后立即刷新域名列表，新域名马上出现在「已绑定域名」里
    await refreshHostnamesOnly();
    return true;
  } catch (err: any) {
    appendLog(`[ERROR] 绑定 DNS 路由失败: ${err}`, 'error', 'server');
    showToast(`绑定失败: ${err}`);
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
    appendLog(`[WARN] 刷新绑定域名失败: ${err}`, 'warn', 'server');
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

// 提交改名：只 PATCH DNS 记录的 name 字段，不改动隧道 ingress 配置
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
    showToast(`${target.hostname} → ${next}`);
    cancelEditDnsRoute();
    await refreshHostnamesOnly();
  } catch (err: any) {
    appendLog(`[ERROR] 修改绑定域名失败: ${err}`, 'error', 'server');
    showToast(`修改域名失败: ${err}`);
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

// 解绑：先删掉该域名的密码锁（锁是挂在域名上的，DNS 记录删掉后就再也定位不到
// 它对应的 Access 应用了），再删 Cloudflare 侧的 CNAME 记录。隧道本身与 ingress 不受影响。
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
        `[WARN] 域名 [${target.hostname}] 的密码锁未能清除，云端可能残留拦截该域名的 Access 应用，可稍后到 Cloudflare 面板手动删除`,
        'warn',
        'server',
      );
    }

    const res = await invoke<string>('delete_dns_route', { recordId: target.recordId });
    appendLog(`[SUCCESS] ${res} (${target.hostname})`, 'success', 'server');
    showToast(
      `${target.hostname} 已解除绑定${done.length ? '，密码锁已一并删除' : ''}`,
    );
    cancelUnbindDnsRoute();
    await refreshHostnamesOnly();
  } catch (err: any) {
    appendLog(`[ERROR] 解除域名绑定失败: ${err}`, 'error', 'server');
    showToast(`解除绑定失败: ${err}`);
  } finally {
    isDnsMutating.value = false;
  }
};

// 与后端对账云端托管隧道：后端用 try_wait() 回收已退出的进程，返回真实在跑的快照
const refreshRemoteTunnels = async (withLog = false) => {
  try {
    const list = await invoke<{ key: string }[]>('list_remote_tunnels');
    remoteRunningKeys.value = list.map(x => x.key);
    // 选中的那条已经被删掉 / 列表里不存在了，就把选中态清掉，避免顶部删除按钮指向幽灵
    if (
      selectedRemoteTunnel.value &&
      !remoteTunnelList.value.some(x => x.id === selectedRemoteTunnel.value?.id)
    ) {
      selectedRemoteTunnel.value = null;
    }
    if (withLog) {
      appendLog(
        `[INFO] 已刷新云端托管隧道列表，共获取到 ${remoteRunningKeys.value.length} 条隧道`,
        'info',
        'remote',
      );
    }
  } catch (err: any) {
    if (withLog) appendLog(`[ERROR] 刷新云端托管隧道列表失败: ${err}`, 'error', 'remote');
  }
};

// 启动指定的那一条云端托管隧道：Token 由后端拿 cert.pem 现取，前端全程不接触 Token
const handleStartRemoteTunnel = async (tunnel: TunnelInfo) => {
  if (isRemoteRunning(tunnel.id)) return;
  soundManager.playSuccess();
  try {
    // 启动成功的日志由 Rust 侧统一广播，这里不再重复打印
    await invoke<string>('start_remote_tunnel_by_id', {
      tunnelId: tunnel.id,
      tunnelName: tunnel.name,
    });
    showToast(`云端托管已启动 (${tunnel.name})`);
    await refreshRemoteTunnels();
    // 隧道刚起来，配置可能已经变了，强刷一次
    void refreshRemoteConfigs({ force: true });
  } catch (err: any) {
    appendLog(`[ERROR] 启动云端托管失败: ${err}`, 'error', 'remote');
    showToast(`${err}`);
  }
};

// 停止指定的那一条云端托管隧道（只停进程，隧道本身不受影响）
const handleStopRemoteTunnel = async (tunnel: TunnelInfo) => {
  soundManager.playClick();
  try {
    // 隧道名一并传给后端：停止日志里显示隧道名，而不是那串 UUID
    await invoke<string>('stop_remote_tunnel', { key: tunnel.id, tunnelName: tunnel.name });
    showToast(`服务端隧道 [${tunnel.name}] 已停止`);
    delete remoteConfigs.value[tunnel.id];
    // 这一条的缓存已经清掉，把整轮缓存判失效，下次切到云端托管会重新补齐
    invalidateRemoteConfigs();
    await refreshRemoteTunnels();
  } catch (err: any) {
    appendLog(`[ERROR] 停止云端托管失败: ${err}`, 'error', 'remote');
  }
};

// 启动临时链接（临时域名）。返回是否启动成功（创建弹窗据此决定是否关闭）。
const handleStartQuick = async (): Promise<boolean> => {
  const port = quickConfig.value.port.trim();
  const protocol = quickConfig.value.protocol;
  const unixSocket = quickConfig.value.unixSocket.trim();

  if (protocol === 'unix' || protocol === 'unix+tls') {
    if (!unixSocket) {
      appendLog(`[ERROR] unix / unix+tls 协议必须填写套接字路径`, 'error', 'quick');
      return false;
    }
  } else if (protocol !== 'hello_world' && !isPortValid(port)) {
    quickPortHasError.value = true;
    appendLog(`[ERROR] 本地端口错误`, 'error', 'quick');
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
    showToast('临时链接已启动，临时域名生成中...');
    return true;
  } catch (err: any) {
    appendLog(`[ERROR] 启动临时链接失败: ${err}`, 'error', 'quick');
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
    appendLog(`[INFO] 已刷新临时链接列表，共获取到 ${merged.length} 条隧道`, 'info', 'quick');
  } catch (err: any) {
    appendLog(`[ERROR] 刷新临时链接列表失败: ${err}`, 'error', 'quick');
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
    showToast('临时链接已停止');
  } catch (err: any) {
    appendLog(`[ERROR] 停止临时链接失败: ${err}`, 'error', 'quick');
  }
};

// 复制临时链接临时链接（点击临时域名触发）
const copyQuickUrl = async (url: string) => {
  try {
    await navigator.clipboard.writeText(url);
    showToast('临时链接已复制到剪贴板');
  } catch {
    appendLog('复制临时链接失败，请检查剪贴板权限', 'error', 'quick');
  }
};

// 复制绑定域名（点击列表里的域名标签触发）
const copyHostname = async (hostname: string) => {
  try {
    await navigator.clipboard.writeText(hostname);
    showToast(`域名已复制: ${hostname}`);
  } catch {
    appendLog(`复制域名失败: ${hostname}（请检查剪贴板权限）`, 'error', 'server');
  }
};

// 通用复制（访问账号 / 访问密码等），成功与否都给一行提示
const copyText = async (text: string) => {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    showToast(t.value.server_tab.copied_toast);
  } catch {
    appendLog('复制失败，请检查剪贴板权限', 'error', 'server');
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
  pendingDelete.value = {
    id: target.id,
    name: target.name,
    scope: 'local',
  };
  showDeleteModal.value = true;
};

// 云端托管列表的删除入口：顶部按钮不带参数（作用于选中行），行内 🗑 直接把 tunnel 传进来
const promptDeleteRemoteTunnel = (tunnel?: TunnelInfo) => {
  const target = tunnel ?? selectedRemoteTunnel.value;
  if (!target) {
    showToast(t.value.server_tab.errors.no_selection);
    return;
  }
  soundManager.playClick();
  selectedRemoteTunnel.value = target;
  pendingDelete.value = { id: target.id, name: target.name, scope: 'remote' };
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
    if (target.scope === 'remote') {
      if (isRemoteRunning(target.id)) {
        await invoke<string>('stop_remote_tunnel', { key: target.id, tunnelName: target.name });
        remoteRunningKeys.value = remoteRunningKeys.value.filter(k => k !== target.id);
      }
    } else if (isTunnelRunning(target.name)) {
      await invoke<string>('stop_server_tunnel', { name: target.name });
      markServerStopped(target.name);
    }
  } catch {
    // 停不掉也不影响强制删除，继续往下走
  }

  const domainNote = bound.length ? `（含 ${bound.length} 条绑定域名）` : '';
  appendLog(`[INFO] 正在强制删除隧道 [${target.name}]${domainNote}...`, 'info', 'server');
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
        appendLog(`[WARN] 残留域名 [${d.hostname}] 清理失败: ${errorText(err)}`, 'warn', 'server');
      }
    }
    if (dnsDeleted) appendLog(`[INFO] 已补删 ${dnsDeleted} 条残留在云端的域名绑定`, 'info', 'server');

    // 2. 密码锁：逐域名删掉云端的 Access 应用 / 策略 / Service Token（含本地记录），
    //    否则域名没了、锁还在云端拦着，软件里也再定位不到它。
    const { done, failed } = await purgeDomainLocks(bound);
    if (failed.length) {
      appendLog(
        `[WARN] 以下域名的密码锁未能清除，云端可能残留 Access 应用，可稍后到 Cloudflare 面板手动删除: ${failed.join('、')}`,
        'warn',
        'server',
      );
    }

    const noteParts: string[] = [];
    if (bound.length || dnsDeleted) noteParts.push(`域名 ${Math.max(bound.length, dnsDeleted)} 条`);
    if (done.length) noteParts.push(`密码锁 ${done.length} 把`);
    showToast(
      `隧道 [${target.name}] 已删除${noteParts.length ? `，已连带清理${noteParts.join('、')}` : ''}`,
    );

    if (selectedTunnel.value?.id === target.id) selectedTunnel.value = null;
    if (selectedRemoteTunnel.value?.id === target.id) selectedRemoteTunnel.value = null;
    delete remoteConfigs.value[target.id];
    await handleRefreshTunnels(target.scope === 'remote' ? 'remote' : 'local');
  } catch (err: any) {
    appendLog(`[ERROR] 删除隧道失败: ${err}`, 'error', 'server');
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
      `[INFO] 已刷新客户端隧道列表，共获取到 ${clientRunningKeys.value.length} 条隧道`,
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
    appendLog(`[ERROR] 客户端域名格式错误`, 'error', 'client');
    return;
  }
  if (!isPortValid(port)) {
    clientFormPortHasError.value = true;
    appendLog(`[ERROR] 本地监听端口错误`, 'error', 'client');
    return;
  }
  if (tokenId.length > 0 !== tokenSecret.length > 0) {
    appendLog(`[ERROR] 访问账号与访问密码必须同时填写或同时留空`, 'error', 'client');
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
      appendLog(`[ERROR] 客户端隧道 [${key}] 已存在，请勿重复`, 'error', 'client');
      showToast('该隧道已存在');
      return;
    }
    const idx = savedClientTunnels.value.findIndex(t => t.key === editingClientKey.value);
    if (idx !== -1) savedClientTunnels.value[idx] = entry;
    persistClientTunnels();
    soundManager.playSuccess();
    appendLog(`[SUCCESS] 客户端隧道配置已更新为 [${domain}:${port}]`, 'success', 'client');
    showToast('客户端隧道已保存');
    showClientAddModal.value = false;
    editingClientKey.value = '';
    return;
  }

  // 新增模式：重复直接拦掉，只落库保存，不自动启动（由用户点行内「启动」再连）
  if (savedClientTunnels.value.some(t => t.key === key)) {
    appendLog(`[ERROR] 客户端隧道 [${key}] 已存在，请勿重复添加`, 'error', 'client');
    showToast('该隧道已存在');
    return;
  }

  savedClientTunnels.value.push(entry);
  persistClientTunnels();
  soundManager.playSuccess();
  appendLog(`[SUCCESS] 客户端隧道配置已保存 [${domain}:${port}]`, 'success', 'client');
  showToast('已保存，点「启动」后连接');
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
    showToast(`客户端隧道 ${row.domain}:${row.port} 已连接`);
  } catch (err: any) {
    appendLog(`[ERROR] 启动客户端隧道失败: ${err}`, 'error', 'client');
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
    showToast(`已断开 ${conn.domain}:${conn.port}`);
  } catch (err: any) {
    appendLog(`[ERROR] 断开客户端连接失败: ${err}`, 'error', 'client');
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
  appendLog(`[INFO] 已删除客户端隧道 [${row.domain}:${row.port}]`, 'warn', 'client');
  showToast(`已删除 ${row.domain}:${row.port}`);
  await refreshClientConnections();
};

// 安装 cloudflared 流程 (根据系统与架构获取官方直链并下载至应用目录)
const handleInstallCloudflared = async () => {
  const target = getCloudflaredTarget();
  appendLog(`[INFO] 检测到当前系统环境: ${target.displayName} (系统: ${target.os}, 架构: ${target.arch})`, 'info', 'misc');
  appendLog(`[INFO] 目标二进制文件: ${target.fileName}`, 'info', 'misc');
  appendLog(`[INFO] 官方直链: ${target.downloadUrl}`, 'info', 'misc');

  isDownloadingCloudflared.value = true;
  try {
    const res = await invoke<string>('download_and_install_cloudflared', {
      downloadUrl: target.downloadUrl,
      filename: target.fileName,
    });
    showToast('已启动 cloudflared 下载，请观察控制台进度');
    appendLog(`[INFO] ${res}`, 'info', 'misc');
  } catch (err: any) {
    appendLog(`[ERROR] 启动安装流程失败: ${err}`, 'error', 'misc');
  } finally {
    isDownloadingCloudflared.value = false;
  }
};

// 杂项操作
const handleCloudflaredLogin = async () => {
  try {
    await invoke<string>('login_cloudflared');
    showToast('已启动 Cloudflared 授权流程');
  } catch (err: any) {
    appendLog(`[ERROR] 启动授权失败: ${err}`, 'error', 'misc');
  }
};

const handleCheckVersion = async () => {
  try {
    const ver = await invoke<string>('check_cloudflared_version');
    appendLog(`[INFO] 当前生效的 Cloudflared 版本: ${ver}`, 'info', 'misc');
    showToast(`版本: ${ver}`);
  } catch (err: any) {
    appendLog(`[ERROR] 检查版本失败: ${err}`, 'error', 'misc');
  }
};

const handleUpdateCloudflared = async () => {
  appendLog('[INFO] 正在检查并更新 cloudflared...', 'info', 'misc');
  try {
    const res = await invoke<string>('update_cloudflared');
    appendLog(`[INFO] 更新结果: ${res}`, 'info', 'misc');
    showToast('Cloudflared 更新检查完成');
  } catch (err: any) {
    appendLog(`[ERROR] 更新失败: ${err}`, 'error', 'misc');
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

// 键盘快捷键与全局点击监听 (ESC 关闭模态窗、下拉菜单与放大预览)
const onKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    if (isLangDropdownOpen.value) {
      isLangDropdownOpen.value = false;
    } else if (showDnsEditModal.value) {
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

const onClickOutside = (e: MouseEvent) => {
  if (langDropdownRef.value && !langDropdownRef.value.contains(e.target as Node)) {
    isLangDropdownOpen.value = false;
  }
};

// 初始化与事件监听
onMounted(async () => {
  syncThemeToDocument();
  document.title = t.value.title;
  window.addEventListener('keydown', onKeyDown);
  document.addEventListener('click', onClickOutside);

  // 获取并监听窗口最大化状态
  try {
    isMaximized.value = await invoke<boolean>('is_window_maximized');
  } catch {}

  // 执行一次输入合法性初步检查（如有初始值）
  if (serverConfig.value.name) onServerNameInput();
  if (serverConfig.value.port) onServerPortInput();
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

    // cloudflared 每次加载配置都会广播这个事件。配置内容现在统一走 Cloudflare API 读，
    // 所以这里只借它判断「这条隧道确实活着」，顺手把运行状态补上。
    await listen<{ key: string; config: string }>('remote-config-update', (event) => {
      const key = event.payload?.key || '';
      if (!key) return;
      if (!remoteRunningKeys.value.includes(key)) remoteRunningKeys.value.push(key);
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
      appendLog(`[SUCCESS] 临时域名已分配: ${url}`, 'success', 'quick');
      showToast('临时域名已生成');
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
    await refreshRemoteTunnels();
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
  document.removeEventListener('click', onClickOutside);
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
  --dropdown-bg: rgba(255, 255, 255, 0.96);
  --dropdown-shadow: 0 10px 30px rgba(0, 0, 0, 0.15), 0 2px 6px rgba(0, 0, 0, 0.08);
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
  --dropdown-bg: rgba(40, 40, 40, 0.96);
  --dropdown-shadow: 0 12px 36px rgba(0, 0, 0, 0.5), 0 2px 8px rgba(0, 0, 0, 0.3);
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

/* 自定义 Win11 优雅平滑语言下拉菜单 (高度统一为 28px, 居中对齐) */
.fluent-dropdown-wrapper {
  position: relative;
  z-index: 5001;
  display: flex;
  align-items: center;
  pointer-events: auto !important;
  -webkit-app-region: no-drag !important;
}

.fluent-dropdown-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 10px;
  box-sizing: border-box;
  background-color: var(--bg-input);
  color: var(--text-primary);
  border: 1px solid var(--border-strong);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer !important;
  outline: none;
  transition: all 0.2s cubic-bezier(0.25, 1, 0.5, 1);
  box-shadow: 0 1px 2px rgba(0,0,0,0.04);
  pointer-events: auto !important;
  -webkit-app-region: no-drag !important;
}

.fluent-dropdown-btn:hover {
  background-color: var(--bg-hover);
  border-color: var(--text-secondary);
}

.fluent-dropdown-btn.open {
  border-color: var(--accent-color);
  box-shadow: 0 0 0 1px var(--accent-color);
}

.dropdown-arrow {
  font-size: 9px;
  color: var(--text-secondary);
  transition: transform 0.25s cubic-bezier(0.25, 1, 0.5, 1);
}

.dropdown-arrow.rotated {
  transform: rotate(180deg);
}

.fluent-dropdown-menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  min-width: 160px;
  background-color: var(--dropdown-bg);
  backdrop-filter: blur(24px);
  border: 1px solid var(--border-strong);
  border-radius: 8px;
  box-shadow: var(--dropdown-shadow);
  padding: 6px;
  z-index: 99999;
}

.dropdown-slide-enter-active,
.dropdown-slide-leave-active {
  transition: opacity 0.22s cubic-bezier(0.25, 1, 0.5, 1),
              transform 0.22s cubic-bezier(0.25, 1, 0.5, 1);
  transform-origin: top right;
}

.dropdown-slide-enter-from,
.dropdown-slide-leave-to {
  opacity: 0;
  transform: translateY(-8px) scale(0.96);
}

.fluent-dropdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 10px;
  border-radius: 6px;
  font-size: 12.5px;
  cursor: pointer;
  transition: all 0.15s ease;
  color: var(--text-primary);
}

.fluent-dropdown-item:hover {
  background-color: var(--bg-hover);
}

.fluent-dropdown-item.active {
  background-color: rgba(96, 205, 255, 0.12);
  color: var(--accent-color);
  font-weight: 600;
}

.item-check {
  margin-left: auto;
  font-size: 12px;
  font-weight: bold;
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

/* 「已绑定域名」管理块：通栏铺满整卡（原五五开两列已取消，表单移入添加弹窗） */
.dns-bound-block {
  min-width: 0;
}

/* 列表铺满整卡宽度，并限制最大高度做内部滚动：列表再长也不会把卡片撑出屏幕。
   上限取约 2 行（表头 34 + 2 行 ≈ 100px），把更多高度让给上方的隧道列表卡（更常用）。 */
.dns-bound-block .fluent-table-wrapper {
  width: 100%;
  max-height: 100px;
}

/* 该表铺满右列。
   注意 1：必须用 .dns-bound-block .fluent-table 这个两级选择器，
   否则会被后面同样只有一级的 .fluent-table { min-width: max-content } 覆盖，
   导致窄框里的表格仍按内容撑宽、只露出第一列并出现横向滚动。
   注意 2：table-layout:fixed 是这里的关键。表格默认按"最长域名"的 min-content
   撑宽，min-width:0 也压不住（实测：框 473px、表 559px、右侧被切 85px）；
   固定布局后列宽只按 width 分配，域名放不下由标签换行 / 省略号处理，不会撑破框。 */
.dns-bound-block .fluent-table {
  width: 100%;
  min-width: 0;
  table-layout: fixed;
}

.dns-bound-table .col-tunnel-name {
  white-space: nowrap;
  /* 定宽 + 省略号：table-layout:fixed 下百分比/1% 会被严格按比例算，
     写成 1% 会把这一列压成一条线（隧道名只剩 "local…"）。
     固定 104px 后剩余宽度（约 370px）全部给域名列，
     正好放得下两个常规域名标签一行（约 343px），不会因为列太窄而多折一行。 */
  width: 104px;
  max-width: 104px;
  overflow: hidden;
  text-overflow: ellipsis;
  vertical-align: top;
}

/* 域名列允许换行：同一隧道的多个域名以标签横向排列，放不下时自然折行 */
.dns-bound-table .col-bound-hostname {
  white-space: normal;
  vertical-align: top;
}

/* ---- 域名标签：一个隧道一行，域名做成小胶囊，改/解绑按钮内嵌在胶囊里 ---- */
.hostname-chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  min-width: 0;
}

.hostname-chip {
  display: inline-flex;
  align-items: center;
  gap: 1px;
  max-width: 100%;
  padding: 1px 3px 1px 8px;
  border-radius: 4px;
  background-color: #005fb8;
  color: #ffffff;
  font-size: 11.5px;
  font-family: 'Consolas', 'Courier New', monospace;
}

.hostname-chip-text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  /* 点击即复制 */
  cursor: pointer;
  user-select: none;
}

.hostname-chip-text:hover {
  text-decoration: underline;
}

.chip-action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 15px;
  height: 15px;
  padding: 0;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: #ffffff;
  font-size: 10px;
  line-height: 1;
  cursor: pointer;
  opacity: 0.8;
  transition: background-color 0.15s ease, opacity 0.15s ease;
}

.chip-action-btn:hover {
  background-color: rgba(255, 255, 255, 0.32);
  opacity: 1;
}

.chip-action-btn.danger:hover {
  background-color: #d13438;
  opacity: 1;
}

/* 域名胶囊内的锁状态标签（已上锁 🔒 / 未上锁） */
.chip-lock-tag {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  flex-shrink: 0;
  padding: 0 4px;
  border-radius: 3px;
  font-size: 10px;
  font-family: inherit;
  background-color: rgba(255, 255, 255, 0.22);
  color: #ffffff;
  white-space: nowrap;
}

.chip-lock-tag.off {
  background-color: rgba(255, 255, 255, 0.14);
  opacity: 0.85;
}

/* 密码锁操作列：每个域名一行，已上锁显示账号/密码+换密码/解锁，未上锁显示「上锁」 */
.dns-bound-table .col-bound-lock {
  white-space: normal;
  vertical-align: top;
  width: 210px;
  max-width: 210px;
}

.lock-chip-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  min-height: 20px;
  padding: 1px 0;
}

/* 锁操作列里的「上锁」按钮：加个浅色底，跟域名胶囊里的动作按钮区分开 */
.chip-action-btn.primary {
  width: auto;
  height: auto;
  padding: 1px 8px;
  border-radius: 4px;
  font-size: 11px;
  color: var(--text-primary, #1a1a1a);
  background-color: var(--chip-lock-btn-bg, #e8f3ff);
  opacity: 1;
  white-space: nowrap;
}

.chip-action-btn.primary:hover {
  background-color: var(--chip-lock-btn-hover, #cfe6ff);
}

.chip-action-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

/* 锁操作列里换密码/解锁按钮：脱离蓝底胶囊，需要自带可见的文字色与悬停底 */
.col-bound-lock .chip-action-btn {
  color: var(--text-primary, #1a1a1a);
  background-color: rgba(0, 0, 0, 0.05);
}

.col-bound-lock .chip-action-btn:hover {
  background-color: rgba(0, 0, 0, 0.12);
}

.col-bound-lock .chip-action-btn.danger {
  color: var(--danger-color, #d13438);
}

.col-bound-lock .chip-action-btn.danger:hover {
  background-color: #d13438;
  color: #ffffff;
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
