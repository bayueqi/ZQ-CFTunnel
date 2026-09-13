export interface TunnelInfo {
  id: string;
  name: string;
  created: string;
  connections: string;
  tunnel_type: 'local' | 'remote';
}

export interface LogEntry {
  id: string;
  timestamp: string;
  message: string;
  level: 'info' | 'warn' | 'error' | 'success';
  source: 'server' | 'client' | 'misc' | 'remote' | 'system';
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
    btn_create: string;
    btn_start: string;
    btn_stop: string;
    btn_refresh: string;
    btn_delete: string;
    btn_clear_log: string;
    status_running: string;
    status_stopped: string;
    mode_local: string;
    mode_remote: string;
    local_list_title: string;
    remote_list_title: string;
    type_local: string;
    type_remote: string;
    dns_section: string;
    dns_tunnel_name: string;
    dns_tunnel_name_placeholder: string;
    dns_domain: string;
    dns_domain_placeholder: string;
    btn_route_dns: string;
    remote_token_label: string;
    remote_token_placeholder: string;
    remote_token_hint: string;
    btn_start_remote: string;
    btn_stop_remote: string;
    status_remote_running: string;
    status_remote_stopped: string;
    remote_config_title: string;
    remote_config_empty: string;
    headers: {
      id: string;
      name: string;
      type: string;
      created: string;
      connections: string;
    };
    errors: {
      tunnel_invalid: string;
      port_invalid: string;
      no_selection: string;
      delete_confirm_title: string;
      delete_confirm_msg: string;
      dns_domain_invalid: string;
    };
  };
  client_tab: {
    domain: string;
    port: string;
    domain_placeholder: string;
    port_placeholder: string;
    btn_connect: string;
    btn_disconnect: string;
    status_connected: string;
    status_disconnected: string;
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
