export interface TunnelInfo {
  id: string;
  name: string;
  created: string;
  connections: string;
  tunnel_type: 'local' | 'remote';
  hostnames?: DnsBinding[];
}

/** 一条指向隧道的 DNS 绑定记录（Cloudflare 区域内的 CNAME）。 */
export interface DnsBinding {
  /** Cloudflare dns_record_id，改名与解绑以此为操作对象 */
  id: string;
  /** 绑定的完整域名，例如 mc.example.com */
  name: string;
}

export interface QuickTunnelItem {
  key: string;
  protocol: string;
  port: string;
  url: string;
  status: 'starting' | 'running';
}

/** 一条正在运行的客户端隧道（后端 cloudflared access tcp 桥接进程）。 */
export interface ClientTunnelItem {
  key: string;
  domain: string;
  port: string;
}

export interface LogEntry {
  id: string;
  timestamp: string;
  message: string;
  level: 'info' | 'warn' | 'error' | 'success';
  source: 'server' | 'client' | 'misc' | 'remote' | 'quick' | 'system';
}

export interface LangPack {
  title: string;
  header: string;

  /** 窗口外壳：标题栏按钮 / 侧边栏 / 控制台把手等非业务文案 */
  chrome: {
    logo_title: string;
    sound_mute_title: string;
    sound_unmute_title: string;
    github_title: string;
    theme_to_light: string;
    theme_to_dark: string;
    btn_minimize: string;
    btn_restore: string;
    btn_maximize: string;
    btn_close: string;
    sidebar_expand: string;
    sidebar_collapse: string;
    sidebar_collapse_text: string;
    console_resize_title: string;
  };
  tabs: {
    server: string;
    client: string;
    misc: string;
  };
  server_tab: {
    tunnel_name: string;
    port: string;
    tunnel_name_placeholder: string;
    port_placeholder: string;
    protocol_label: string;
    protocol_http: string;
    protocol_https: string;
    protocol_tcp: string;
    protocol_ssh: string;
    protocol_rdp: string;
    protocol_smb: string;
    protocol_unix: string;
    protocol_unix_tls: string;
    protocol_hello_world: string;
    unix_socket_label: string;
    unix_socket_placeholder: string;
    quick_no_address_hint: string;
    quick_no_address_label: string;
    btn_create: string;
    btn_start: string;
    btn_stop: string;
    btn_refresh: string;
    btn_refreshing: string;
    btn_delete: string;
    btn_clear_log: string;
    status_running: string;
    status_stopped: string;
    status_not_running: string;
    mode_local: string;
    mode_remote: string;
    sub_mode_quick: string;
    sub_mode_named: string;
    nav_quick: string;
    nav_named: string;
    nav_remote: string;
    quick_port_label: string;
    quick_port_placeholder: string;
    btn_generate_quick: string;
    btn_stop_quick: string;
    quick_url_title: string;
    quick_url_empty: string;
    quick_list_title: string;
    quick_list_empty: string;
    quick_stop: string;
    click_to_copy: string;
    btn_open: string;
    hostname_unbound: string;
    local_list_title: string;
    remote_list_title: string;
    type_local: string;
    type_remote: string;
    dns_section: string;
    dns_tunnel_name: string;
    dns_domain: string;
    dns_domain_placeholder: string;
    btn_route_dns: string;
    dns_bound_title: string;
    dns_bound_empty: string;
    /** 隧道分组标题里的域名计数，{count} 替换为实际条数 */
    dns_group_count: string;
    dns_col_tunnel: string;
    dns_add_btn: string;
    /** 域名行内「改名」按钮 */
    dns_btn_rename: string;
    /** 域名行内「解绑」按钮 */
    dns_btn_unbind: string;
    dns_add_title: string;
    dns_edit_title: string;
    dns_edit_label: string;
    btn_unbind: string;
    btn_save: string;
    remote_token_label: string;
    remote_token_placeholder: string;
    remote_token_hint: string;
    btn_start_remote: string;
    btn_stop_remote: string;
    status_remote_running: string;
    status_remote_stopped: string;
    remote_config_title: string;
    remote_config_empty: string;
    config_sec_published: string;
    config_sec_hostname: string;
    config_sec_cidr: string;
    config_none: string;
    config_load_failed: string;
    delete_force_hint: string;
    remote_add_btn: string;
    remote_add_title: string;
    remote_edit_title: string;
    remote_col_tunnel: string;
    remote_col_status: string;
    remote_col_action: string;
    remote_empty: string;
    remote_btn_edit: string;
    remote_btn_delete: string;
    remote_delete_confirm_title: string;
    remote_delete_confirm_msg: string;
    remote_token_invalid: string;
    remote_token_dup: string;
    // ===== 隧道密码锁（Cloudflare Access） =====
    btn_lock: string;
    btn_unlock: string;
    named_create_title: string;
    quick_create_title: string;
    named_edit_title: string;
    named_create_domain_label: string;
    named_create_domain_placeholder: string;
/** 兜底行下拉里的默认档：未匹配的请求直接回 404 */
    protocol_catch_all_404: string;
/** 协议下拉里代表「原始 service」（无法反解成协议+端口）的那一项 */
    protocol_raw: string;
/** 修改弹窗打开后正在拉取云端配置 */
    form_loading_config: string;
/** 云端配置拉取失败时缀在原因前面的标题 */
    form_load_failed: string;
/** 修改态下隧道名只读的说明 */
    form_name_readonly_hint: string;
/** raw 行的 service 原文输入框标题 */
    form_service_label: string;
/** 一条路由的域名输入框标题 */
    form_hostname_label: string;
/** 域名输入框占位文案 */
    form_route_hostname_placeholder: string;
/** 「添加路由」按钮 */
    form_add_route: string;
/** 行内删除本条路由按钮 */
    form_remove_route: string;
/** 末尾兜底规则行的标签 */
    form_catch_all_label: string;
/** 末尾兜底行下面那行说明（它接住所有未匹配的请求） */
    form_catch_all_hint: string;
/** 主机名路由区块的说明 */
    form_host_routes_hint: string;
/** CIDR 路由区块的说明 */
    form_cidr_routes_hint: string;
/** 「添加主机名路由」按钮 */
    form_add_host_route: string;
/** 「添加 CIDR 路由」按钮 */
    form_add_cidr_route: string;
/** 主机名路由的主机名输入框标题 */
    form_host_label: string;
/** CIDR 路由的网段输入框标题 */
    form_cidr_label: string;
/** 路由备注输入框标题 */
    form_comment_label: string;
/** 保存成功提示 */
    form_saved: string;
    lock_switch_label: string;
    lock_switch_hint: string;
    lock_need_domain: string;
    lock_host_label: string;
    lock_success_title: string;
    lock_success_msg: string;
    lock_account_label: string;
    lock_secret_label: string;
    lock_done_btn: string;
    unlock_confirm_title: string;
    unlock_confirm_msg: string;
    unlocked_toast: string;
    /** 锁状态徽标：已上锁 */
    lock_on: string;
    /** 锁状态徽标：未上锁 */
    lock_off: string;
    /** 凭据掩码状态下「显示凭据」按钮 */
    lock_cred_show: string;
    /** 凭据展开状态下「隐藏凭据」按钮 */
    lock_cred_hide: string;
    /** 「换密码」按钮 */
    btn_rotate_password: string;
    /** 换密码二次确认弹窗标题 */
    rotate_confirm_title: string;
    /** 换密码二次确认弹窗正文，{hostname} 会被替换为域名 */
    rotate_confirm_msg: string;
    /** 删除隧道确认框里的连带删除提示，{domains} / {locks} 会被替换为数量 */
    delete_cascade_hint: string;
    /** 解绑域名确认框里的提示：该域名的密码锁会一并删除 */
    dns_unbind_lock_hint: string;
    /** 改域名弹窗里的提示：旧域名的密码锁会被删除，新域名需重新上锁 */
    dns_rename_lock_hint: string;
    copied_toast: string;
    edit_need_config: string;
    edit_restart_hint: string;
    /** 临时隧道行内状态徽标：已运行 */
    quick_status_online: string;
    /** 临时隧道行内状态徽标：正在申请临时域名 */
    quick_status_starting: string;
    /** 固定隧道表与客户端表在刷新中的空表文案 */
    table_refreshing: string;
    /** 固定隧道表与客户端表的空表文案 */
    table_empty: string;
    headers: {
      id: string;
      name: string;
      type: string;
      created: string;
      connections: string;
      hostname: string;
      status: string;
      actions: string;
      password: string;
      /** DNS 路由绑定表里「访问密码锁」列的表头 */
      lock: string;
    };
    errors: {
      tunnel_invalid: string;
      port_invalid: string;
      no_selection: string;
      delete_confirm_title: string;
      delete_confirm_msg: string;
      dns_domain_invalid: string;
      err_service_required: string;
      err_socket_required: string;
      err_port_required: string;
      err_hostname_invalid: string;
      err_hostname_dup: string;
      err_host_required: string;
      err_cidr_required: string;
      dns_unbind_confirm_title: string;
      dns_unbind_confirm_msg: string;
      quick_stop_confirm_title: string;
      quick_stop_confirm_msg: string;
      token_pair_invalid: string;
    };
  };
  client_tab: {
    title: string;
    domain: string;
    port: string;
    domain_placeholder: string;
    port_placeholder: string;
    btn_connect: string;
    btn_disconnect: string;
    status_connected: string;
    status_disconnected: string;
    add_btn: string;
    add_title: string;
    col_domain: string;
    col_port: string;
    col_password: string;
    col_status: string;
    col_action: string;
    empty: string;
    refresh_btn: string;
    btn_start: string;
    btn_stop: string;
    btn_edit: string;
    btn_delete: string;
    btn_save: string;
    delete_confirm_title: string;
    delete_confirm_msg: string;
    edit_title: string;
    // 访问凭据（目标隧道开了密码锁时成对填写）
    token_id_label: string;
    token_id_placeholder: string;
    token_secret_label: string;
    token_secret_placeholder: string;
    errors: {
      domain_invalid: string;
      port_invalid: string;
      token_pair_invalid: string;
    };
  };
  misc_tab: {
    btn_install: string;
    btn_install_desc: string;
    btn_open_config_dir: string;
    btn_open_config_dir_desc: string;
    config_dir_warning: string;
    btn_login: string;
    btn_check_version: string;
    btn_update: string;
    btn_download: string;
    // 访问密码锁凭证（Cloudflare Access API 用）
    access_token_label: string;
    access_token_placeholder: string;
    access_token_hint: string;
    access_token_saved: string;
    /** 授权登录卡片说明 */
    btn_login_desc: string;
    /** 检查版本卡片说明 */
    btn_check_version_desc: string;
    /** 在线更新卡片说明 */
    btn_update_desc: string;
    /** 前往官方下载页卡片说明 */
    btn_download_desc: string;
  };
  exit_modal: {
    title: string;
    message: string;
    btn_confirm: string;
    btn_cancel: string;
  };
  console: {
    title: string;
    filter_all: string;
    btn_clear: string;
    btn_copy: string;
    copy_logs_success: string;
    empty: string;
  };
  /**
   * 日志与吐司文案。桌面端由 App.vue 用 t.value.logs.* 取用；
   * 网页演示模式（utils/tauriBridge.ts）没有 Vue 上下文，经 i18n 的 activePack() 取用。
   * 带 {xxx} 的占位符由 i18n 的 fmt() 填充。
   */
  logs: {
    toast_sound_on: string;
    toast_sound_off: string;
    target_hello_world: string;
    host_route_deleted: string;
    host_route_delete_failed: string;
    host_route_delete_failed_named: string;
    host_route_create_failed: string;
    cidr_route_deleted: string;
    cidr_route_delete_failed: string;
    cidr_route_save_failed: string;
    route_invalid_at: string;
    tunnel_created: string;
    err_tunnel_id_missing: string;
    err_tunnel_id_not_found: string;
    ingress_written: string;
    blank_host_rows_skipped: string;
    dns_route_create_failed: string;
    dns_route_delete_failed: string;
    dns_failed_removed_from_ingress: string;
    tunnel_save_failed: string;
    access_token_saved_log: string;
    lock_ok: string;
    lock_failed: string;
    unlock_failed: string;
    rotate_ok: string;
    rotate_failed: string;
    leftover_check_failed: string;
    app_ready: string;
    copy_logs_failed: string;
    opened_in_browser: string;
    open_link_failed: string;
    config_dir_opened: string;
    config_dir_open_failed: string;
    tunnel_list_refreshed: string;
    tunnel_list_refresh_failed: string;
    tunnel_need_config: string;
    unix_socket_required: string;
    tunnel_port_invalid: string;
    tunnel_started: string;
    tunnel_start_failed: string;
    no_tunnel_to_stop: string;
    tunnel_stopped: string;
    tunnel_stop_failed: string;
    no_tunnel_for_dns: string;
    dns_bound_ok: string;
    dns_bind_failed: string;
    bind_failed_toast: string;
    refresh_hostnames_failed: string;
    rename_lock_purge_failed: string;
    rename_ok: string;
    rename_ok_lock_purged: string;
    rename_domain_failed: string;
    rename_domain_failed_toast: string;
    unbind_lock_purge_failed: string;
    unbind_ok: string;
    unbind_ok_lock_purged: string;
    unbind_ingress_removed: string;
    unbind_ingress_remove_failed: string;
    unbind_ok_ingress_removed: string;
    unbind_domain_failed: string;
    unbind_domain_failed_toast: string;
    quick_port_invalid: string;
    quick_started: string;
    quick_start_failed: string;
    quick_list_refreshed: string;
    quick_list_refresh_failed: string;
    quick_stopped: string;
    quick_stop_failed: string;
    quick_url_copied: string;
    quick_url_copy_failed: string;
    hostname_copied: string;
    hostname_copy_failed: string;
    copy_failed_clipboard: string;
    delete_bound_note: string;
    force_deleting_tunnel: string;
    leftover_domain_cleanup_failed: string;
    leftover_dns_deleted: string;
    delete_lock_purge_failed: string;
    cascade_domains: string;
    cascade_locks: string;
    tunnel_deleted: string;
    cascade_join: string;
    cascade_note: string;
    tunnel_delete_failed: string;
    client_list_refreshed: string;
    client_domain_invalid: string;
    client_port_invalid: string;
    client_token_pair_invalid: string;
    client_dup_edit: string;
    client_dup_add: string;
    client_dup_toast: string;
    client_updated: string;
    client_saved: string;
    client_created: string;
    client_saved_hint: string;
    client_connected: string;
    client_connect_failed: string;
    client_disconnected: string;
    client_disconnect_failed: string;
    client_deleted: string;
    client_deleted_toast: string;
    detected_env: string;
    target_binary: string;
    official_url: string;
    download_started: string;
    install_failed: string;
    login_started: string;
    login_failed: string;
    version_current: string;
    version_toast: string;
    version_check_failed: string;
    updating_cloudflared: string;
    update_result: string;
    update_done: string;
    update_failed: string;
    quick_domain_assigned: string;
    quick_domain_ready: string;
    demo_tunnel_created_cred: string;
    demo_tunnel_created: string;
    demo_tunnel_deleted: string;
    demo_tunnel_deleted_toast: string;
    demo_server_connecting: string;
    demo_server_proxy_up: string;
    demo_server_connected: string;
    demo_server_started: string;
    demo_server_closed: string;
    demo_server_stopped_toast: string;
    demo_client_already: string;
    demo_client_connecting: string;
    demo_client_proxy_up: string;
    demo_client_connected: string;
    demo_client_not_running: string;
    demo_client_closed: string;
    demo_domain_invalid: string;
    demo_lock_ok: string;
    demo_unlock_ok: string;
    demo_unlock_ok_toast: string;
    demo_rotate_ok: string;
    demo_missing_tunnel_id: string;
    demo_remote_already: string;
    demo_remote_token: string;
    demo_remote_connected: string;
    demo_default_config: string;
    demo_remote_started: string;
    demo_server_not_running: string;
    demo_server_closed_named: string;
    demo_tunnel_id_invalid: string;
    demo_data: string;
    demo_all: string;
    demo_ingress_written: string;
    demo_config_updated: string;
    demo_host_created: string;
    demo_host_created_toast: string;
    demo_host_deleted: string;
    demo_host_deleted_toast: string;
    demo_cidr_created: string;
    demo_cidr_created_toast: string;
    demo_cidr_updated: string;
    demo_cidr_updated_toast: string;
    demo_cidr_deleted: string;
    demo_cidr_deleted_toast: string;
    demo_dns_bound: string;
    demo_dns_bound_toast: string;
    demo_dns_renamed: string;
    demo_dns_renamed_toast: string;
    demo_dns_unbound: string;
    demo_dns_unbound_toast: string;
    demo_quick_requesting: string;
    demo_quick_started: string;
    demo_quick_stopped: string;
    demo_quick_stopped_toast: string;
    demo_checking_version: string;
    demo_version_latest: string;
    demo_version_latest_toast: string;
    demo_downloading: string;
    demo_download_progress: string;
    demo_download_done: string;
    demo_download_started: string;
    demo_login_page: string;
    demo_login_page_toast: string;
    demo_open_config_dir: string;
    demo_exit_hint: string;
  };
}
