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
    dns_col_tunnel: string;
    dns_add_btn: string;
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
    headers: {
      id: string;
      name: string;
      type: string;
      created: string;
      connections: string;
      hostname: string;
      status: string;
      actions: string;
    };
    errors: {
      tunnel_invalid: string;
      port_invalid: string;
      no_selection: string;
      delete_confirm_title: string;
      delete_confirm_msg: string;
      dns_domain_invalid: string;
      dns_unbind_confirm_title: string;
      dns_unbind_confirm_msg: string;
      quick_stop_confirm_title: string;
      quick_stop_confirm_msg: string;
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
    errors: {
      domain_invalid: string;
      port_invalid: string;
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
