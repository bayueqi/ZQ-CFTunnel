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

    <!-- Win11 选项卡导航 (带平滑横向滑动指示滑块) -->
    <nav class="fluent-nav-tabs" ref="navTabsRef">
      <!-- 平滑滑动背景指示滑块 -->
      <div class="nav-tab-slider" :style="tabSliderStyle"></div>

      <button
        ref="serverTabRef"
        :class="['nav-tab', { active: currentTab === 'server' }]"
        @click="switchTab('server')"
      >
        <span class="tab-icon">🖥️</span>
        <span class="tab-text">{{ t.tabs.server }}</span>
        <span v-if="serverRunning" class="status-dot green"></span>
      </button>

      <button
        ref="clientTabRef"
        :class="['nav-tab', { active: currentTab === 'client' }]"
        @click="switchTab('client')"
      >
        <span class="tab-icon">💻</span>
        <span class="tab-text">{{ t.tabs.client }}</span>
        <span v-if="clientRunning" class="status-dot green"></span>
      </button>

      <button
        ref="miscTabRef"
        :class="['nav-tab', { active: currentTab === 'misc' }]"
        @click="switchTab('misc')"
      >
        <span class="tab-icon">⚙️</span>
        <span class="tab-text">{{ t.tabs.misc }}</span>
      </button>
    </nav>

    <!-- 主体内容卡片区 (平滑过渡动效) -->
    <main class="fluent-body">
      <!-- 1. 服务端 Tab -->
      <section v-show="currentTab === 'server'" class="tab-view server-view animated-view">
        <!-- 本地 / 远程 切换 -->
        <div class="mode-switch-bar">
          <button
            :class="['mode-switch-btn', { active: serverMode === 'local' }]"
            @click="switchServerMode('local')"
          >
            <span class="mode-dot local"></span>
            {{ t.server_tab.mode_local }}
          </button>
          <button
            :class="['mode-switch-btn', { active: serverMode === 'remote' }]"
            @click="switchServerMode('remote')"
          >
            <span class="mode-dot remote"></span>
            {{ t.server_tab.mode_remote }}
          </button>
        </div>

        <!-- ============ 本地隧道视图 ============ -->
        <div v-show="serverMode === 'local'" class="server-sub-view">
          <!-- 输入表单卡片 -->
          <div class="fluent-card form-card">
            <div class="form-grid">
              <!-- 隧道名字 -->
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
                  />
                </div>
                <div v-if="serverNameHasError" class="error-tip">
                  <span class="error-icon">⚠️</span>
                  {{ t.server_tab.errors.tunnel_invalid }}
                </div>
              </div>

              <!-- 端口号 -->
              <div class="fluent-form-group">
                <label class="form-label">
                  {{ t.server_tab.port }}
                  <span class="required">*</span>
                </label>
                <div class="input-container">
                  <input
                    type="text"
                    v-model="serverConfig.port"
                    :placeholder="t.server_tab.port_placeholder"
                    :class="['fluent-input', { 'input-error': serverPortHasError }]"
                    @input="onServerPortInput"
                  />
                </div>
                <div v-if="serverPortHasError" class="error-tip">
                  <span class="error-icon">⚠️</span>
                  {{ t.server_tab.errors.port_invalid }}
                </div>
              </div>

              <!-- 协议选择 -->
              <div class="fluent-form-group">
                <label class="form-label">
                  {{ t.server_tab.protocol_label }}
                </label>
                <div class="input-container">
                  <select
                    v-model="serverConfig.protocol"
                    class="fluent-input fluent-select"
                  >
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
            </div>

            <!-- Unix 套接字路径（仅 unix / unix+tls 协议时显示） -->
            <div class="form-grid" v-if="serverConfig.protocol === 'unix' || serverConfig.protocol === 'unix+tls'">
              <div class="fluent-form-group">
                <label class="form-label">
                  {{ t.server_tab.unix_socket_label }}
                  <span class="required">*</span>
                </label>
                <div class="input-container">
                  <input
                    type="text"
                    v-model="serverConfig.unixSocket"
                    :placeholder="t.server_tab.unix_socket_placeholder"
                    class="fluent-input"
                  />
                </div>
              </div>
            </div>

            <!-- 本地隧道主要操作按钮 -->
            <div class="actions-row center-actions">
              <button class="fluent-btn" @click="handleCreateTunnel" :disabled="isCreatingTunnel">
                <span class="btn-icon">➕</span>
                {{ t.server_tab.btn_create }}
              </button>

              <button
                v-if="!serverRunning"
                class="fluent-btn primary"
                @click="handleStartServer"
                :disabled="!canStartServer"
              >
                <span class="btn-icon">▶</span>
                {{ t.server_tab.btn_start }}
              </button>

              <button
                v-else
                class="fluent-btn danger"
                @click="handleStopServer"
              >
                <span class="btn-icon">⏹</span>
                {{ t.server_tab.btn_stop }}
              </button>

              <div class="status-pill" :class="serverRunning ? 'online' : 'offline'">
                <span class="pill-dot"></span>
                {{ serverRunning ? t.server_tab.status_running : t.server_tab.status_stopped }}
              </div>
            </div>
          </div>

          <!-- 本地隧道列表卡片 -->
          <div class="fluent-card table-card">
            <div class="card-header">
              <h3 class="card-title">{{ t.server_tab.local_list_title }}</h3>
              <span class="card-subtitle">（双击行可快速填入隧道名字）</span>
            </div>

            <div class="fluent-table-wrapper">
              <table class="fluent-table">
                <thead>
                  <tr>
                    <th class="col-id">{{ t.server_tab.headers.id }}</th>
                    <th class="col-name">{{ t.server_tab.headers.name }}</th>
                    <th class="col-type">{{ t.server_tab.headers.type }}</th>
                    <th class="col-created">{{ t.server_tab.headers.created }}</th>
                    <th class="col-connections">{{ t.server_tab.headers.connections }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="tunnel in localTunnelList"
                    :key="tunnel.id"
                    :class="{ selected: selectedTunnel?.id === tunnel.id }"
                    @click="selectTunnel(tunnel)"
                    @dblclick="onTunnelDoubleClick(tunnel)"
                  >
                    <td class="col-id mono" :title="tunnel.id">{{ tunnel.id }}</td>
                    <td class="col-name font-bold">{{ tunnel.name }}</td>
                    <td class="col-type">
                      <span class="type-badge type-local">{{ t.server_tab.type_local }}</span>
                    </td>
                    <td class="col-created mono">{{ tunnel.created }}</td>
                    <td class="col-connections">{{ tunnel.connections || '-' }}</td>
                  </tr>
                  <tr v-if="localTunnelList.length === 0">
                    <td colspan="5" class="empty-table">
                      {{ isRefreshingTunnels ? '正在刷新列表...' : '未发现本地隧道' }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>

            <div class="actions-row table-actions">
              <button class="fluent-btn" @click="handleRefreshTunnels" :disabled="isRefreshingTunnels">
                <span class="btn-icon">🔄</span>
                {{ t.server_tab.btn_refresh }}
              </button>

              <button
                class="fluent-btn danger-outline"
                @click="promptDeleteTunnel"
                :disabled="!selectedTunnel"
              >
                <span class="btn-icon">🗑️</span>
                {{ t.server_tab.btn_delete }}
              </button>
            </div>
          </div>

          <!-- DNS 路由绑定卡片 -->
          <div class="fluent-card form-card dns-route-card">
            <h3 class="card-title dns-route-title">{{ t.server_tab.dns_section }}</h3>
            <div class="form-grid">
              <div class="fluent-form-group">
                <label class="form-label">
                  {{ t.server_tab.dns_tunnel_name }}
                  <span class="required">*</span>
                </label>
                <div class="input-container">
                  <input
                    type="text"
                    v-model="dnsRoute.name"
                    :placeholder="t.server_tab.dns_tunnel_name_placeholder"
                    :class="['fluent-input', { 'input-error': dnsRouteNameHasError }]"
                    @input="onDnsRouteNameInput"
                  />
                </div>
                <div v-if="dnsRouteNameHasError" class="error-tip">
                  <span class="error-icon">⚠️</span>
                  {{ t.server_tab.errors.tunnel_invalid }}
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
                  />
                </div>
                <div v-if="dnsRouteDomainHasError" class="error-tip">
                  <span class="error-icon">⚠️</span>
                  {{ t.server_tab.errors.dns_domain_invalid }}
                </div>
              </div>
            </div>

            <div class="actions-row center-actions">
              <button
                class="fluent-btn primary"
                @click="handleRouteDns"
                :disabled="dnsRouteNameHasError || dnsRouteDomainHasError || !dnsRoute.name || !dnsRoute.domain"
              >
                <span class="btn-icon">🔗</span>
                {{ t.server_tab.btn_route_dns }}
              </button>
            </div>
          </div>
        </div>

        <!-- ============ 远程隧道视图 ============ -->
        <div v-show="serverMode === 'remote'" class="server-sub-view">
          <!-- Token 输入 + 启动/停止 -->
          <div class="fluent-card form-card">
            <h3 class="card-title">{{ t.server_tab.remote_token_label }}</h3>
            <div class="form-grid">
              <div class="fluent-form-group">
                <div class="input-container">
                  <input
                    type="text"
                    v-model="remoteToken"
                    :placeholder="t.server_tab.remote_token_placeholder"
                    class="fluent-input"
                  />
                </div>
              </div>
            </div>

            <div class="actions-row center-actions">
              <button
                v-if="!remoteRunning"
                class="fluent-btn primary"
                @click="handleStartRemoteTunnel"
                :disabled="!remoteToken.trim()"
              >
                <span class="btn-icon">▶</span>
                {{ t.server_tab.btn_start_remote }}
              </button>

              <button
                v-else
                class="fluent-btn danger"
                @click="handleStopRemoteTunnel"
              >
                <span class="btn-icon">⏹</span>
                {{ t.server_tab.btn_stop_remote }}
              </button>

              <div class="status-pill" :class="remoteRunning ? 'online' : 'offline'">
                <span class="pill-dot"></span>
                {{ remoteRunning ? t.server_tab.status_remote_running : t.server_tab.status_remote_stopped }}
              </div>
            </div>
          </div>

          <!-- 远程隧道列表卡片 -->
          <div class="fluent-card table-card">
            <div class="card-header">
              <h3 class="card-title">{{ t.server_tab.remote_list_title }}</h3>
            </div>

            <div class="fluent-table-wrapper">
              <table class="fluent-table">
                <thead>
                  <tr>
                    <th class="col-id">{{ t.server_tab.headers.id }}</th>
                    <th class="col-name">{{ t.server_tab.headers.name }}</th>
                    <th class="col-type">{{ t.server_tab.headers.type }}</th>
                    <th class="col-created">{{ t.server_tab.headers.created }}</th>
                    <th class="col-connections">{{ t.server_tab.headers.connections }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="tunnel in remoteTunnelList"
                    :key="tunnel.id"
                  >
                    <td class="col-id mono" :title="tunnel.id">{{ tunnel.id }}</td>
                    <td class="col-name font-bold">{{ tunnel.name }}</td>
                    <td class="col-type">
                      <span class="type-badge type-remote">{{ t.server_tab.type_remote }}</span>
                    </td>
                    <td class="col-created mono">{{ tunnel.created }}</td>
                    <td class="col-connections">{{ tunnel.connections || '-' }}</td>
                  </tr>
                  <tr v-if="remoteTunnelList.length === 0">
                    <td colspan="5" class="empty-table">
                      {{ isRefreshingTunnels ? '正在刷新列表...' : '未发现远程隧道' }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>

            <div class="actions-row table-actions">
              <button class="fluent-btn" @click="handleRefreshTunnels" :disabled="isRefreshingTunnels">
                <span class="btn-icon">🔄</span>
                {{ t.server_tab.btn_refresh }}
              </button>
            </div>
          </div>

          <!-- 云端 ingress 配置卡片 -->
          <div class="fluent-card form-card remote-config-card">
            <h3 class="card-title">{{ t.server_tab.remote_config_title }}</h3>
            <div class="remote-config-body">
              <pre v-if="remoteConfigText">{{ remoteConfigText }}</pre>
              <div v-else class="remote-config-empty">{{ t.server_tab.remote_config_empty }}</div>
            </div>
          </div>
        </div>
      </section>

      <!-- 2. 客户端 Tab (无额外滑动条) -->
      <section v-show="currentTab === 'client'" class="tab-view client-view animated-view">
        <div class="fluent-card form-card">
          <div class="form-grid">
            <!-- 隧道域名 -->
            <div class="fluent-form-group">
              <label class="form-label">
                {{ t.client_tab.domain }}
                <span class="required">*</span>
              </label>
              <div class="input-container">
                <input
                  type="text"
                  v-model="clientConfig.domain"
                  :placeholder="t.client_tab.domain_placeholder"
                  :class="['fluent-input', { 'input-error': clientDomainHasError }]"
                  @input="onClientDomainInput"
                />
              </div>
              <div v-if="clientDomainHasError" class="error-tip">
                <span class="error-icon">⚠️</span>
                {{ t.client_tab.errors.domain_invalid }}
              </div>
            </div>

            <!-- 本地监听端口 -->
            <div class="fluent-form-group">
              <label class="form-label">
                {{ t.client_tab.port }}
                <span class="required">*</span>
              </label>
              <div class="input-container">
                <input
                  type="text"
                  v-model="clientConfig.port"
                  :placeholder="t.client_tab.port_placeholder"
                  :class="['fluent-input', { 'input-error': clientPortHasError }]"
                  @input="onClientPortInput"
                />
              </div>
              <div v-if="clientPortHasError" class="error-tip">
                <span class="error-icon">⚠️</span>
                {{ t.client_tab.errors.port_invalid }}
              </div>
            </div>
          </div>

          <!-- 客户端主要操作按钮 (居中) -->
          <div class="actions-row center-actions">
            <button
              v-if="!clientRunning"
              class="fluent-btn primary large"
              @click="handleStartClient"
              :disabled="clientDomainHasError || clientPortHasError || !clientConfig.domain || !clientConfig.port"
            >
              <span class="btn-icon">🔗</span>
              {{ t.client_tab.btn_connect }}
            </button>

            <button
              v-else
              class="fluent-btn danger large"
              @click="handleStopClient"
            >
              <span class="btn-icon">🔌</span>
              {{ t.client_tab.btn_disconnect }}
            </button>

            <div class="status-pill" :class="clientRunning ? 'online' : 'offline'">
              <span class="pill-dot"></span>
              {{ clientRunning ? t.client_tab.status_connected : t.client_tab.status_disconnected }}
            </div>
          </div>
        </div>
      </section>

      <!-- 3. 杂项 Tab (保留上下滑动 Slider) -->
      <section v-show="currentTab === 'misc'" class="tab-view misc-view animated-view">
        <!-- 快捷操作区 -->
        <div class="fluent-card action-tiles-card">
          <div class="tile-grid">
            <!-- 1. 安装 cloudflared 按钮 -->
            <button class="tile-btn highlight-tile" @click="handleInstallCloudflared" :disabled="isDownloadingCloudflared">
              <span class="tile-icon">📦</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_install || '安装 cloudflared' }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_install_desc || '自动检测操作系统与CPU架构并下载至应用目录' }}</span>
              </div>
            </button>

            <!-- 2. 打开本地配置文件目录 -->
            <button class="tile-btn highlight-folder-tile" @click="handleOpenConfigDir">
              <span class="tile-icon">📂</span>
              <div class="tile-info">
                <span class="tile-title">{{ t.misc_tab.btn_open_config_dir || '打开本地配置文件目录' }}</span>
                <span class="tile-desc">{{ t.misc_tab.btn_open_config_dir_desc || '在文件资源管理器中查看 ~/.cloudflared 配置文件目录' }}</span>
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

    <!-- Win11 确认删除隧道模态弹窗 -->
    <div v-if="showDeleteModal" class="fluent-modal-overlay" @click.self="cancelDelete">
      <div class="fluent-modal-dialog">
        <div class="modal-header">
          <h3 class="modal-title">⚠️ {{ t.server_tab.errors.delete_confirm_title }}</h3>
        </div>
        <div class="modal-body">
          <p>{{ t.server_tab.errors.delete_confirm_msg.replace('{name}', selectedTunnel?.name || '') }}</p>
        </div>
        <div class="modal-footer">
          <button class="fluent-btn" @click="cancelDelete">{{ t.exit_modal.btn_cancel }}</button>
          <button class="fluent-btn danger" @click="confirmDeleteTunnel">确认删除</button>
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
import { LangKey, TunnelInfo, LogEntry } from './types';
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
soundManager.enabled = isSoundEnabled.value;

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
  soundManager.enabled = isSoundEnabled.value;
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
  // 切换语言后立即强制刷新页面，让 Vue 重新挂载整个 DOM，彻底防止错乱
  window.location.reload();
};

// 选项卡状态与平滑滑动指示器
const currentTab = ref(localStorage.getItem('app_tab') || 'server');
const navTabsRef = ref<HTMLElement | null>(null);
const serverTabRef = ref<HTMLButtonElement | null>(null);
const clientTabRef = ref<HTMLButtonElement | null>(null);
const miscTabRef = ref<HTMLButtonElement | null>(null);

const tabSliderStyle = ref({
  left: '0px',
  width: '0px',
  opacity: '0',
});

const updateTabSlider = () => {
  nextTick(() => {
    let targetEl: HTMLButtonElement | null = null;
    if (currentTab.value === 'server') targetEl = serverTabRef.value;
    else if (currentTab.value === 'client') targetEl = clientTabRef.value;
    else if (currentTab.value === 'misc') targetEl = miscTabRef.value;

    if (targetEl && navTabsRef.value) {
      const navRect = navTabsRef.value.getBoundingClientRect();
      const elRect = targetEl.getBoundingClientRect();
      const left = elRect.left - navRect.left;
      const width = elRect.width;

      tabSliderStyle.value = {
        left: `${left}px`,
        width: `${width}px`,
        opacity: '1',
      };
    }
  });
};

// 表单输入
const serverConfig = ref({
  name: localStorage.getItem('server_tunnel_name') || 'mc',
  port: localStorage.getItem('server_port') || '25565',
  protocol: localStorage.getItem('server_protocol') || 'http',
  unixSocket: localStorage.getItem('server_unix_socket') || '',
});

const clientConfig = ref({
  domain: localStorage.getItem('client_domain') || '',
  port: localStorage.getItem('client_port') || '25566',
});

// DNS 路由绑定表单 (cloudflared tunnel route dns)
const dnsRoute = ref({
  name: localStorage.getItem('dns_route_name') || serverConfig.value.name || '',
  domain: localStorage.getItem('dns_route_domain') || '',
});

// 输入错误校验状态
const serverNameHasError = ref(false);
const serverPortHasError = ref(false);
const clientDomainHasError = ref(false);
const clientPortHasError = ref(false);
const dnsRouteNameHasError = ref(false);
const dnsRouteDomainHasError = ref(false);

// 判断服务端表单是否满足启动条件（hello_world 无需端口，unix 协议需要套接字路径）
const canStartServer = computed(() => {
  const p = serverConfig.value.protocol;
  if (!serverConfig.value.name) return false;
  if (p === 'hello_world') return !serverNameHasError.value;
  if (p === 'unix' || p === 'unix+tls') {
    return !serverNameHasError.value && !!serverConfig.value.unixSocket.trim();
  }
  return !serverNameHasError.value && !serverPortHasError.value && !!serverConfig.value.port;
});

// 运行状态
const serverRunning = ref(false);
const clientRunning = ref(false);
const isRefreshingTunnels = ref(false);
const isCreatingTunnel = ref(false);
const isDownloadingCloudflared = ref(false);

// 服务端模式：本地 / 远程 切换
const serverMode = ref<'local' | 'remote'>(localStorage.getItem('server_mode') === 'remote' ? 'remote' : 'local');

const switchServerMode = (mode: 'local' | 'remote') => {
  serverMode.value = mode;
  localStorage.setItem('server_mode', mode);
};

// 远程隧道
const remoteToken = ref(localStorage.getItem('remote_token') || '');
const remoteRunning = ref(false);
const remoteConfigText = ref('');

// 隧道列表与选中项
const tunnelList = ref<TunnelInfo[]>([]);
const selectedTunnel = ref<TunnelInfo | null>(null);

const localTunnelList = computed(() => tunnelList.value.filter(t => t.tunnel_type === 'local'));
const remoteTunnelList = computed(() => tunnelList.value.filter(t => t.tunnel_type === 'remote'));

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

const onClientDomainInput = () => {
  const val = clientConfig.value.domain;
  clientDomainHasError.value = val.length > 0 && !isDomainValid(val);
  localStorage.setItem('client_domain', val);
};

const onClientPortInput = () => {
  const val = clientConfig.value.port;
  clientPortHasError.value = val.length > 0 && !isPortValid(val);
  localStorage.setItem('client_port', val);
};

const onDnsRouteNameInput = () => {
  const val = dnsRoute.value.name;
  dnsRouteNameHasError.value = val.length > 0 && !isTunnelNameValid(val);
  localStorage.setItem('dns_route_name', val);
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
  localStorage.setItem('app_tab', tab);
  updateTabSlider();
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

// 追加日志
const appendLog = (message: string, level: LogEntry['level'] = 'info', source: LogEntry['source'] = 'system') => {
  logs.value.push({
    id: `${Date.now()}-${Math.random().toString(36).substr(2, 6)}`,
    timestamp: new Date().toLocaleTimeString(),
    message,
    level,
    source,
  });

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

// 打开本地配置文件目录 (~/.cloudflared)
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

// 双击隧道填充名字
const onTunnelDoubleClick = (tunnel: TunnelInfo) => {
  selectedTunnel.value = tunnel;
  serverConfig.value.name = tunnel.name;
  onServerNameInput();
  appendLog(`[INFO] 已选择并填充隧道 [${tunnel.name}] (ID: ${tunnel.id})`, 'info', 'server');
  showToast(`已选择隧道: ${tunnel.name}`);
};

// 刷新隧道列表
const handleRefreshTunnels = async () => {
  isRefreshingTunnels.value = true;
  try {
    const res = await invoke<TunnelInfo[]>('list_tunnels');
    tunnelList.value = res;
    appendLog(`[INFO] 已刷新隧道列表，共获取到 ${res.length} 条隧道`, 'info', 'server');
  } catch (err: any) {
    appendLog(`[ERROR] 刷新隧道列表失败: ${err}`, 'error', 'server');
  } finally {
    isRefreshingTunnels.value = false;
  }
};

// 创建隧道 (触发 playSuccess 音效)
const handleCreateTunnel = async () => {
  const name = serverConfig.value.name.trim();
  if (!name || !isTunnelNameValid(name)) {
    serverNameHasError.value = true;
    appendLog(`[ERROR] ${t.value.server_tab.errors.tunnel_invalid}`, 'error', 'server');
    return;
  }

  soundManager.playSuccess();
  isCreatingTunnel.value = true;
  try {
    const res = await invoke<string>('create_tunnel', { name });
    appendLog(`[SUCCESS] 成功创建隧道 [${name}]: ${res}`, 'success', 'server');
    showToast(`隧道 [${name}] 创建成功！`);
    await handleRefreshTunnels();
  } catch (err: any) {
    appendLog(`[ERROR] 创建隧道失败: ${err}`, 'error', 'server');
  } finally {
    isCreatingTunnel.value = false;
  }
};

// 启动服务端隧道 (触发 playSuccess 音效)
const handleStartServer = async () => {
  const name = serverConfig.value.name.trim();
  const port = serverConfig.value.port.trim();
  const protocol = serverConfig.value.protocol;
  const unixSocket = serverConfig.value.unixSocket.trim();

  if (!isTunnelNameValid(name)) {
    serverNameHasError.value = true;
    appendLog(`[ERROR] 隧道名错误`, 'error', 'server');
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

  soundManager.playSuccess();
  try {
    const res = await invoke<string>('start_server_tunnel', { name, port, protocol, unixSocket });
    localStorage.setItem('server_tunnel_name', name);
    localStorage.setItem('server_port', port);
    localStorage.setItem('server_protocol', protocol);
    localStorage.setItem('server_unix_socket', unixSocket);
    serverRunning.value = true;
    appendLog(`[SUCCESS] ${res}`, 'success', 'server');
    const desc = protocol === 'hello_world'
      ? 'hello_world 内置测试服务器'
      : protocol === 'unix' || protocol === 'unix+tls'
        ? `${protocol}:${unixSocket}`
        : `${protocol}://127.0.0.1:${port}`;
    showToast(`隧道 [${name}] 已启动 (${desc})`);
  } catch (err: any) {
    appendLog(`[ERROR] 启动服务端隧道失败: ${err}`, 'error', 'server');
  }
};

// 停止服务端隧道 (普通点击音效)
const handleStopServer = async () => {
  soundManager.playClick();
  try {
    const res = await invoke<string>('stop_server_tunnel');
    serverRunning.value = false;
    appendLog(`[INFO] ${res}`, 'warn', 'server');
    showToast(`服务端隧道已停止`);
  } catch (err: any) {
    appendLog(`[ERROR] 停止服务端失败: ${err}`, 'error', 'server');
  }
};

// 绑定 DNS 路由 (cloudflared tunnel route dns <name> <hostname>)
const handleRouteDns = async () => {
  const name = dnsRoute.value.name.trim();
  const domain = dnsRoute.value.domain.trim();

  if (!isTunnelNameValid(name)) {
    dnsRouteNameHasError.value = true;
    appendLog(`[ERROR] ${t.value.server_tab.errors.tunnel_invalid}`, 'error', 'server');
    return;
  }
  if (!isDomainValid(domain)) {
    dnsRouteDomainHasError.value = true;
    appendLog(`[ERROR] ${t.value.server_tab.errors.dns_domain_invalid}`, 'error', 'server');
    return;
  }

  soundManager.playSuccess();
  try {
    const res = await invoke<string>('route_dns_tunnel', { name, hostname: domain });
    appendLog(`[SUCCESS] ${res}`, 'success', 'server');
    showToast(`DNS 路由绑定成功: ${domain} → ${name}`);
  } catch (err: any) {
    appendLog(`[ERROR] 绑定 DNS 路由失败: ${err}`, 'error', 'server');
  }
};

// 启动远程隧道 (tunnel run --token，临时运行)
const handleStartRemoteTunnel = async () => {
  const token = remoteToken.value.trim();
  if (!token) {
    appendLog(`[ERROR] 请先粘贴远程隧道 Token`, 'error', 'remote');
    return;
  }
  localStorage.setItem('remote_token', token);
  remoteConfigText.value = '';
  soundManager.playSuccess();
  try {
    const res = await invoke<string>('start_remote_tunnel', { token });
    remoteRunning.value = true;
    appendLog(`[SUCCESS] ${res}`, 'success', 'remote');
    showToast('远程隧道已启动');
  } catch (err: any) {
    appendLog(`[ERROR] 启动远程隧道失败: ${err}`, 'error', 'remote');
  }
};

// 停止远程隧道
const handleStopRemoteTunnel = async () => {
  soundManager.playClick();
  try {
    const res = await invoke<string>('stop_remote_tunnel');
    remoteRunning.value = false;
    appendLog(`[INFO] ${res}`, 'warn', 'remote');
    showToast('远程隧道已停止');
  } catch (err: any) {
    appendLog(`[ERROR] 停止远程隧道失败: ${err}`, 'error', 'remote');
  }
};

// 删除隧道确认流程
const promptDeleteTunnel = () => {
  if (!selectedTunnel.value) {
    showToast(t.value.server_tab.errors.no_selection);
    return;
  }
  showDeleteModal.value = true;
};

const cancelDelete = () => {
  showDeleteModal.value = false;
};

const confirmDeleteTunnel = async () => {
  if (!selectedTunnel.value) return;
  const tunnelName = selectedTunnel.value.name;
  showDeleteModal.value = false;

  appendLog(`[INFO] 正在删除隧道 [${tunnelName}]...`, 'info', 'server');
  try {
    const res = await invoke<string>('delete_tunnel', { name: tunnelName });
    appendLog(`[SUCCESS] ${res}`, 'success', 'server');
    showToast(`隧道 [${tunnelName}] 已删除`);
    selectedTunnel.value = null;
    await handleRefreshTunnels();
  } catch (err: any) {
    appendLog(`[ERROR] 删除隧道失败: ${err}`, 'error', 'server');
  }
};

// 启动客户端连接 (触发 playSuccess 音效)
const handleStartClient = async () => {
  const domain = clientConfig.value.domain.trim();
  const port = clientConfig.value.port.trim();

  if (!isDomainValid(domain)) {
    clientDomainHasError.value = true;
    appendLog(`[ERROR] 客户端域名格式错误`, 'error', 'client');
    return;
  }
  if (!isPortValid(port)) {
    clientPortHasError.value = true;
    appendLog(`[ERROR] 本地监听端口错误`, 'error', 'client');
    return;
  }

  soundManager.playSuccess();
  try {
    const res = await invoke<string>('start_client_tunnel', { domain, port });
    clientRunning.value = true;
    appendLog(`[SUCCESS] ${res}`, 'success', 'client');
    showToast(`客户端隧道已连接 (端口: ${port})`);
  } catch (err: any) {
    appendLog(`[ERROR] 客户端连接失败: ${err}`, 'error', 'client');
  }
};

// 断开客户端连接 (普通点击音效)
const handleStopClient = async () => {
  soundManager.playClick();
  try {
    const res = await invoke<string>('stop_client_tunnel');
    clientRunning.value = false;
    appendLog(`[INFO] ${res}`, 'warn', 'client');
    showToast(`客户端已断开连接`);
  } catch (err: any) {
    appendLog(`[ERROR] 断开客户端连接失败: ${err}`, 'error', 'client');
  }
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
  window.addEventListener('resize', updateTabSlider);

  updateTabSlider();

  // 获取并监听窗口最大化状态
  try {
    isMaximized.value = await invoke<boolean>('is_window_maximized');
  } catch {}

  // 执行一次输入合法性初步检查（如有初始值）
  if (serverConfig.value.name) onServerNameInput();
  if (serverConfig.value.port) onServerPortInput();
  if (clientConfig.value.domain) onClientDomainInput();
  if (clientConfig.value.port) onClientPortInput();

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

    // 监听远程隧道云端 ingress 配置更新
    await listen<string>('remote-config-update', (event) => {
      remoteConfigText.value = event.payload;
    });
  } catch (e) {
    console.error('Listen event error:', e);
  }

  // 异步获取初始隧道列表与状态
  try {
    serverRunning.value = await invoke<boolean>('is_server_running');
    clientRunning.value = await invoke<boolean>('is_client_running');
    remoteRunning.value = await invoke<boolean>('is_remote_running');
    await handleRefreshTunnels();
  } catch {}
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
  document.removeEventListener('click', onClickOutside);
  window.removeEventListener('resize', updateTabSlider);
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
  --console-bg: #181818;
  --console-text: #d4d4d4;
  --console-header-bg: #222222;
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
  --console-bg: #141414;
  --console-text: #d4d4d4;
  --console-header-bg: #1f1f1f;
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

/* 选项卡导航 (带平滑横向滑动指示器) */
.fluent-nav-tabs {
  position: relative;
  z-index: 10;
  display: flex;
  gap: 6px;
  padding: 6px 16px 0;
  background-color: var(--bg-tab-bar);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

/* 平滑滑动背景指示滑块 */
.nav-tab-slider {
  position: absolute;
  bottom: 0;
  height: calc(100% - 6px);
  background-color: var(--bg-app);
  border: 1px solid var(--border-subtle);
  border-bottom: none;
  border-radius: 6px 6px 0 0;
  transition: left 0.28s cubic-bezier(0.25, 1, 0.5, 1),
              width 0.28s cubic-bezier(0.25, 1, 0.5, 1),
              opacity 0.2s ease;
  z-index: 1;
  pointer-events: none;
}

.nav-tab-slider::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 20%;
  right: 20%;
  height: 3px;
  background-color: var(--accent-color);
  border-radius: 3px 3px 0 0;
}

.nav-tab {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 18px;
  background: transparent;
  border: 1px solid transparent;
  border-bottom: none;
  border-radius: 6px 6px 0 0;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: color 0.2s ease;
  z-index: 2;
}

.nav-tab:hover {
  color: var(--text-primary);
}

.nav-tab.active {
  color: var(--text-primary);
  font-weight: 600;
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
}

/* DNS 路由绑定卡片 (位于服务端本地视图) */
.dns-route-card {
  flex-shrink: 0;
}

.dns-route-title {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

/* 本地/远程切换栏 */
.mode-switch-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 14px;
  flex-shrink: 0;
}

.mode-switch-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 18px;
  border: 1px solid var(--border-strong);
  border-radius: 8px;
  background-color: var(--bg-input);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.mode-switch-btn:hover {
  background-color: var(--bg-hover);
}

.mode-switch-btn.active {
  background-color: var(--accent-color);
  border-color: var(--accent-color);
  color: var(--accent-text);
}

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

/* 子视图容器 */
.server-sub-view {
  display: flex;
  flex-direction: column;
  gap: 14px;
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
  margin-bottom: 14px;
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
  flex: 1;
  min-height: 140px;
  overflow: hidden;
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

.tile-btn.highlight-tile {
  border-color: var(--accent-color);
  background: linear-gradient(135deg, var(--bg-card-solid) 0%, rgba(96, 205, 255, 0.08) 100%);
}

.tile-btn.highlight-folder-tile {
  border-color: rgba(0, 95, 184, 0.4);
  background: linear-gradient(135deg, var(--bg-card-solid) 0%, rgba(255, 185, 0, 0.08) 100%);
}

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
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  user-select: none;
}

.console-title-area {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  color: #eaeaea;
}

.log-count {
  font-size: 11px;
  color: #888888;
}

.console-actions {
  display: flex;
  gap: 8px;
}

.console-btn {
  padding: 2px 8px;
  background: transparent;
  color: #cccccc;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 4px;
  font-size: 11px;
  cursor: pointer;
  user-select: none;
}

.console-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
  color: #ffffff;
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
  color: #666666;
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
  color: #777777;
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

.log-info .log-tag { color: #60cdff; }
.log-info .log-msg { color: #d4d4d4; }

.log-warn .log-tag { color: #fce100; }
.log-warn .log-msg { color: #ffe666; }

.log-error .log-tag { color: #ff99a4; }
.log-error .log-msg { color: #ff99a4; }

.log-success .log-tag { color: #6ccb5f; }
.log-success .log-msg { color: #98e68e; }

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
