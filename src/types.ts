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

export type LangKey = 'zh_CN' | 'zh_TW' | 'pt_BR' | 'es_ES' | 'en_US' | 'ja_JP';

export interface LangPack {
  title: string;
  header: string;
  lang_button: string;
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
      err_catch_all_last: string;
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
}
