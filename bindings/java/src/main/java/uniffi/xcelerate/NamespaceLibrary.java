package uniffi.xcelerate;


final class NamespaceLibrary {
    static synchronized String findLibraryName(String componentName) {
        String libOverride = System.getProperty("uniffi.component." + componentName + ".libraryOverride");
        if (libOverride != null) {
            return libOverride;
        }
        return "xcelerate";
    }

    static java.lang.foreign.SymbolLookup loadLibrary() {
        String name = findLibraryName("xcelerate");
        if (name.startsWith("/") // Unix absolute path
                || name.startsWith("\\\\") // Windows UNC path
                || (name.length() > 2 && name.charAt(1) == ':')) // Windows drive path (e.g. C:\)
        {
            System.load(name);
        } else {
            System.loadLibrary(name);
        }
        return java.lang.foreign.SymbolLookup.loaderLookup();
    }

    static void uniffiCheckContractApiVersion() {
        int bindingsContractVersion = 30;
        int scaffoldingContractVersion = UniffiLib.ffi_xcelerate_uniffi_contract_version();
        if (bindingsContractVersion != scaffoldingContractVersion) {
            throw new RuntimeException("UniFFI contract version mismatch: try cleaning and rebuilding your project");
        }
    }
    static void uniffiCheckApiChecksums() {
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_audit_log() != ((short) 37952)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_audit_verify() != ((short) 5412)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_available_plugins() != ((short) 9431)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_browser_contexts() != ((short) 50137)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_capabilities() != ((short) 7431)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_close() != ((short) 831)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_cookies() != ((short) 36914)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_delete_cookie() != ((short) 14366)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_event_names() != ((short) 44664)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_grant_permissions() != ((short) 57820)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_is_connected() != ((short) 10958)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_listens_to() != ((short) 12245)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_load_plugin() != ((short) 5693)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_new_context() != ((short) 28184)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_new_page() != ((short) 31633)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_on() != ((short) 2255)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_once() != ((short) 22376)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_plugin() != ((short) 11907)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_plugin_names() != ((short) 58296)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_remove_all_listeners() != ((short) 51158)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_remove_listener() != ((short) 41339)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_reset_permissions() != ((short) 21496)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_set_cookie() != ((short) 6259)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_set_download_behavior() != ((short) 23198)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_start_tracing() != ((short) 14885)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_stop_tracing() != ((short) 57049)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_targets() != ((short) 28936)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_use_plugin() != ((short) 20462)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_user_agent() != ((short) 20558)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_version() != ((short) 64817)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_wait_for_event() != ((short) 20884)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_wait_for_event_default() != ((short) 53096)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_browser_ws_endpoint() != ((short) 36520)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_attribute() != ((short) 8836)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_call_bool() != ((short) 19329)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_call_json() != ((short) 56720)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_call_on_selector() != ((short) 20589)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_call_on_selector_all() != ((short) 58660)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_call_string() != ((short) 1191)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_click() != ((short) 26136)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_click_stealth() != ((short) 64888)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_count() != ((short) 40137)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_dispose() != ((short) 27134)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_evaluate_bool() != ((short) 62708)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_evaluate_handle() != ((short) 34826)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_evaluate_json() != ((short) 20134)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_evaluate_string() != ((short) 15210)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_focus() != ((short) 34225)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_get_by_label() != ((short) 41888)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_get_by_role() != ((short) 15624)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_get_by_text() != ((short) 11298)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_get_properties() != ((short) 28646)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_hover() != ((short) 32638)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_hover_stealth() != ((short) 12397)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_inner_html() != ((short) 63319)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_press() != ((short) 13244)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_query_selector() != ((short) 59248)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_query_selector_all() != ((short) 57750)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_query_selector_attr() != ((short) 63681)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_query_selector_xpath() != ((short) 47775)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_screenshot() != ((short) 55082)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_screenshot_base64() != ((short) 62387)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_select_option() != ((short) 21736)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_set_input_files() != ((short) 32784)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_text() != ((short) 41314)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_type_text() != ((short) 45944)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_element_wait_for_selector() != ((short) 53340)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_activate() != ((short) 30852)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_activate_target() != ((short) 6362)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document() != ((short) 20123)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_add_style_tag() != ((short) 9947)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_authenticate() != ((short) 15669)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_bring_to_front() != ((short) 14186)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_call_bool() != ((short) 62656)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_call_json() != ((short) 37033)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_call_on_selector() != ((short) 27164)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_call_on_selector_all() != ((short) 24130)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_call_string() != ((short) 28160)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_clear_requests() != ((short) 25306)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_click_mouse() != ((short) 54243)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_close() != ((short) 53159)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_content() != ((short) 15096)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_cookie() != ((short) 45979)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_cookies() != ((short) 45327)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_coverage_start_css() != ((short) 860)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_coverage_start_js() != ((short) 4186)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_coverage_stop_css() != ((short) 59476)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_coverage_stop_js() != ((short) 12341)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_create_pdf_stream() != ((short) 54525)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_decode_base64() != ((short) 39526)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_emulate_idle_state() != ((short) 53017)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_emulate_media() != ((short) 27664)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_ensure_interception() != ((short) 6857)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_evaluate_bool() != ((short) 12902)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_evaluate_handle() != ((short) 57739)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_evaluate_json() != ((short) 10653)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_evaluate_string() != ((short) 6817)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_event_names() != ((short) 15640)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_execute_cdp_cmd() != ((short) 20070)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_find_element() != ((short) 4260)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_frame() != ((short) 33986)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_frame_name() != ((short) 1668)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_frames() != ((short) 13809)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_get_by_label() != ((short) 63689)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_get_by_role() != ((short) 32218)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_get_by_text() != ((short) 27497)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_get_default_timeout() != ((short) 57791)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_go_back() != ((short) 60849)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_go_forward() != ((short) 725)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_handle_js_dialog() != ((short) 6178)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_inject_file() != ((short) 16195)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled() != ((short) 59945)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_keyboard_down() != ((short) 53133)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_keyboard_press() != ((short) 43593)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_keyboard_type() != ((short) 53179)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_keyboard_up() != ((short) 24408)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_listens_to() != ((short) 42886)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_main_frame() != ((short) 10285)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_metrics() != ((short) 16960)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_mouse_down() != ((short) 52368)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_mouse_up() != ((short) 52299)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_move_mouse() != ((short) 17586)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_navigate() != ((short) 51495)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_on() != ((short) 15602)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_once() != ((short) 33401)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_pdf() != ((short) 50825)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_press() != ((short) 21741)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_query_selector_all() != ((short) 64158)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_query_selector_xpath() != ((short) 48442)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_raw_window_bounds() != ((short) 13012)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_reload() != ((short) 20867)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_remove_all_listeners() != ((short) 31887)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_remove_listener() != ((short) 35484)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_remove_script() != ((short) 13802)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_request() != ((short) 58954)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_requests() != ((short) 29432)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_route() != ((short) 24223)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_route_abort() != ((short) 32588)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_route_from_har() != ((short) 9232)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_route_fulfill() != ((short) 29300)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_screenshot() != ((short) 62867)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_screenshot_full() != ((short) 56180)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_select_option() != ((short) 5310)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_cache_enabled() != ((short) 36286)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_content() != ((short) 60133)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_default_timeout() != ((short) 58523)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_drag_interception() != ((short) 35102)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_emulated_media_features() != ((short) 49723)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_extra_http_headers() != ((short) 43902)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_input_files() != ((short) 1582)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_javascript_enabled() != ((short) 56309)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_offline() != ((short) 37394)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_request_interception() != ((short) 47016)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_storage_state() != ((short) 55671)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_user_agent() != ((short) 65506)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_viewport_size() != ((short) 47964)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_window_bounds() != ((short) 34825)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_window_position() != ((short) 55844)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_window_size() != ((short) 39920)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_set_window_state() != ((short) 29849)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_start_screencast() != ((short) 3179)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_start_tracing() != ((short) 44969)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_stop_screencast() != ((short) 14965)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_stop_tracing() != ((short) 507)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_storage_state() != ((short) 8033)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_target_id() != ((short) 16602)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_title() != ((short) 57758)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_touch_tap() != ((short) 50285)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_unroute() != ((short) 49265)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_unroute_all() != ((short) 21098)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_url() != ((short) 13992)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_wait_for_event() != ((short) 16279)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_wait_for_event_default() != ((short) 30417)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_wait_for_function() != ((short) 39925)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_wait_for_navigation() != ((short) 28813)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_wait_for_selector() != ((short) 58306)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_wait_for_xpath() != ((short) 14726)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_window_id() != ((short) 52015)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_window_position() != ((short) 23821)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_window_rect() != ((short) 55898)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_page_window_size() != ((short) 35222)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_pluginhandle_invoke() != ((short) 29371)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_pluginhandle_ops() != ((short) 11713)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_method_pluginhandle_plugin_name() != ((short) 61258)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
        if (UniffiLib.uniffi_xcelerate_checksum_constructor_browser_launch() != ((short) 45323)) {
            throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
        }
    }
}

// Define FFI callback types
