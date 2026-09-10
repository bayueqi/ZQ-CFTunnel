export interface TunnelInfo {
  id: string;
  name: string;
  created: string;
  connections: string;
  tunnel_type: string;
}

export interface LogEntry {
  id: string;
  timestamp: string;
  message: string;
  level: 'info' | 'warn' | 'error' | 'success';
  source: 'server' | 'client' | 'misc' | 'system';
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
    btn_create: string;
    btn_start: string;
    btn_stop: string;
    btn_refresh: string;
    btn_delete: string;
    btn_clear_log: string;
    status_running: string;
    status_stopped: string;
    list_title: string;
    tunnel_type_local: string;
    tunnel_type_remote: string;
    headers: {
      id: string;
      name: string;
      created: string;
      connections: string;
      type: string;
    };
    errors: {
      tunnel_invalid: string;
      port_invalid: string;
      no_selection: string;
      delete_confirm_title: string;
      delete_confirm_msg: string;
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
    remote_section: string;
    remote_token: string;
    remote_token_placeholder: string;
    remote_token_hint: string;
    btn_install_service: string;
    btn_uninstall_service: string;
    btn_start_service: string;
    btn_stop_service: string;
    status_service_running: string;
    status_service_stopped: string;
    errors: {
      token_invalid: string;
    };
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
