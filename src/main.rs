use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button, Label, ListBox, ListBoxRow, ScrolledWindow, Orientation, PolicyType, Align};
use std::rc::Rc;
use std::cell::RefCell;
use asthetic_clipboard::{ClipboardStorage, AppConfig};

const APP_ID: &str = "com.asthetic.clipboard";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        // Toggle Logic: If window exists, toggle it. Else build it.
        if let Some(window) = app.active_window() {
             if window.is_visible() {
                 window.close();
             } else {
                 window.present();
             }
        } else {
            build_ui(app);
        }
    });

    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Asthetic Clipboard")
        .default_width(400)
        .default_height(600)
        // .decorated(false) // Custom header bar provides decoration
        .modal(true)
        .build();

    // Data Dependencies (Initialize Early)
    let menu_counter = Rc::new(RefCell::new(0));
    let storage = Rc::new(RefCell::new(ClipboardStorage::load().unwrap_or_default()));
    let config = Rc::new(RefCell::new(AppConfig::load().unwrap_or_default()));
    let list_box = Rc::new(ListBox::new());
    list_box.add_css_class("content-list");
    list_box.set_selection_mode(gtk4::SelectionMode::None);

    // --- Settings Logic (Moved Up) ---
    // Settings Button
    let settings_btn = gtk4::MenuButton::new();
    settings_btn.set_icon_name("emblem-system-symbolic"); // Gear icon usually
    settings_btn.add_css_class("flat");
    settings_btn.set_halign(Align::End); // Inside header, alignment might matter less if packed

    // Settings Menu Popover
    let settings_popover = gtk4::Popover::new();
    
    // Track menu state for settings to prevent window close
    let menu_counter_settings_map = menu_counter.clone();
    settings_popover.connect_map(move |_| {
        *menu_counter_settings_map.borrow_mut() += 1;
    });
    let menu_counter_settings_unmap = menu_counter.clone();
    settings_popover.connect_unmap(move |_| {
        let mut c = menu_counter_settings_unmap.borrow_mut();
        if *c > 0 { *c -= 1; }
    });

    let settings_box = gtk4::Box::new(Orientation::Vertical, 10);
    settings_box.set_margin_top(10);
    settings_box.set_margin_bottom(10);
    settings_box.set_margin_start(10);
    settings_box.set_margin_end(10);

    // 1. Theme Toggle
    let theme_box = gtk4::Box::new(Orientation::Horizontal, 10);
    let theme_label = Label::new(Some("Dark Mode"));
    let theme_switch = gtk4::Switch::new();
    theme_switch.set_active(config.borrow().theme == "dark");
    
    // Apply initial theme
    // Logic handled by load_css at end of build_ui

    let config_theme = config.clone();
    theme_switch.connect_state_set(move |_, state| {
        let theme_str = if state { "dark" } else { "light" };
        config_theme.borrow_mut().theme = theme_str.to_string();
        let _ = config_theme.borrow().save();
        
        load_css(state);
        
        glib::Propagation::Proceed
    });
    theme_box.append(&theme_label);
    theme_box.append(&theme_switch);
    settings_box.append(&theme_box);

    // 2. History Limit
    let limit_box = gtk4::Box::new(Orientation::Horizontal, 10);
    let limit_label = Label::new(Some("History Limit"));
    let limit_spin = gtk4::SpinButton::with_range(10.0, 500.0, 10.0);
    limit_spin.set_value(50.0); // Default
    // Load config logic later
    limit_box.append(&limit_label);
    limit_box.append(&limit_spin);
    settings_box.append(&limit_box);
    
    // 3. Start at Login
    let start_box = gtk4::Box::new(Orientation::Horizontal, 10);
    let start_label = Label::new(Some("Start at Login"));
    let start_switch = gtk4::Switch::new();
    // Load config logic later
    start_box.append(&start_label);
    start_box.append(&start_switch);
    settings_box.append(&start_box);

    let sep = gtk4::Separator::new(Orientation::Horizontal);
    settings_box.append(&sep);

    // 4. Global Actions (Moved from per-item)
    // Clear Unpinned
    let clear_unpinned_btn = Button::with_label("Clear Unpinned");
    clear_unpinned_btn.add_css_class("menu-button");
    let storage_clear_unpinned = storage.clone();
    let list_box_clear_unpinned = list_box.clone();
    let window_clear_unpinned = window.clone();
    let menu_counter_clear_unpinned = menu_counter.clone();
    clear_unpinned_btn.connect_clicked(move |_| {
        {
            let mut s = storage_clear_unpinned.borrow_mut();
            s.history.retain(|i| i.pinned);
            let _ = s.save();
        }
        refresh_list(&list_box_clear_unpinned, &storage_clear_unpinned.borrow(), &window_clear_unpinned, storage_clear_unpinned.clone(), menu_counter_clear_unpinned.clone());
    });
    settings_box.append(&clear_unpinned_btn);

    // Clear All
    let clear_all_btn = Button::with_label("Clear All");
    clear_all_btn.add_css_class("menu-button");
    clear_all_btn.add_css_class("destructive-action");
    let storage_clear_all = storage.clone();
    let list_box_clear_all = list_box.clone();
    let window_clear_all = window.clone();
    let menu_counter_clear_all = menu_counter.clone();
    clear_all_btn.connect_clicked(move |_| {
        {
            let mut s = storage_clear_all.borrow_mut();
            s.history.clear();
            let _ = s.save();
        }
        refresh_list(&list_box_clear_all, &storage_clear_all.borrow(), &window_clear_all, storage_clear_all.clone(), menu_counter_clear_all.clone());
    });
    settings_box.append(&clear_all_btn);

    settings_popover.set_child(Some(&settings_box));
    settings_btn.set_popover(Some(&settings_popover));

    // --- Header Implementation ---
    // Custom Header Bar (CSD)
    let header_bar = gtk4::HeaderBar::new();
    header_bar.set_show_title_buttons(true);
    // Layout: "close" means ONLY show the close button at the end (right side usually) from content
    header_bar.set_decoration_layout(Some(":close")); 

    // Add Settings Button to Header (Left)
    header_bar.pack_start(&settings_btn);

    window.set_titlebar(Some(&header_bar));

    // --- Window Logic ---
    // Close on blur (click outside) - with debounce to handle popover focus switching
    let win_weak = window.downgrade();
    let menu_counter_blur = menu_counter.clone();
    window.connect_is_active_notify(move |win| {
        let is_active = win.is_active();
        
        if !is_active {
            let win_weak_inner = win_weak.clone();
            let menu_counter_inner = menu_counter_blur.clone();
            
            // Wait slightly to see if focus was just transferred to our own popover
            // Increased to 300ms to handle sluggish focus transfers
            glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                if let Some(w) = win_weak_inner.upgrade() {
                    let open_menus = *menu_counter_inner.borrow();
                    
                    // Just check if we are still inactive AND no menus are open
                    if !w.is_active() && open_menus == 0 {
                        w.close();
                    }
                }
            });
        }
    });

    let vbox = gtk4::Box::new(Orientation::Vertical, 5);
    window.set_child(Some(&vbox));

    // Scrolled Window for List (using list_box created earlier)
    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .min_content_height(400)
        .vexpand(true)
        .build();
    vbox.append(&scrolled_window);
    scrolled_window.set_child(Some(list_box.as_ref()));

    // Footer Removed!

    refresh_list(&list_box, &storage.borrow(), &window, storage.clone(), menu_counter.clone());

    // Initial CSS Load
    load_css(config.borrow().theme == "dark");

    window.present();
}

fn refresh_list(
    list_box: &ListBox, 
    storage: &ClipboardStorage, 
    window: &ApplicationWindow, 
    storage_rc: Rc<RefCell<ClipboardStorage>>,
    menu_counter: Rc<RefCell<usize>>
) {
    // Clear existing children
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    for (i, item) in storage.history.iter().enumerate() {
        let row = ListBoxRow::new();
        row.set_activatable(false); // Important: stop listbox from handling activation

        let hbox = gtk4::Box::new(Orientation::Horizontal, 10);
        hbox.set_margin_top(5);
        hbox.set_margin_bottom(5);
        hbox.set_margin_start(5);
        hbox.set_margin_end(5);

        // Pin Icon (if pinned)
        if item.pinned {
            let pin_icon = gtk4::Image::from_icon_name("emblem-favorite-symbolic");
            pin_icon.add_css_class("pinned");
            pin_icon.set_pixel_size(16); // Small icon
            pin_icon.set_valign(Align::Center);
            hbox.append(&pin_icon);
        }

        // Content Area
        let content_box = gtk4::Box::new(Orientation::Horizontal, 0);
        content_box.set_hexpand(true);
        
        let item_content = item.content.clone();
        let item_kind = item.kind.clone();
        
        if item.kind == "image" {
             // Render Image
             let picture = gtk4::Picture::for_filename(&item.content);
             picture.set_content_fit(gtk4::ContentFit::Contain);
             picture.set_height_request(100); // Thumbnail size
             picture.set_halign(Align::Start);
             content_box.append(&picture);
        } else {
             // Render Text with "Show More" logic
             let display_content = item.content.trim();
             let is_long = display_content.chars().count() > 500;
             let display_text = if is_long {
                 display_content.chars().take(500).collect::<String>() + "..."
             } else {
                 display_content.to_string()
             };

             // Vertical container for Content + Button
             let content_vbox = gtk4::Box::new(Orientation::Vertical, 5);
             content_vbox.set_hexpand(true);

             // Text Area
             let text_event_box = gtk4::Box::new(Orientation::Horizontal, 0);
             text_event_box.set_hexpand(true);
             
             let content_label = Label::new(Some(&display_text));
             content_label.set_hexpand(true);
             content_label.set_halign(Align::Start);
             content_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
             content_label.set_lines(10); 
             content_label.set_wrap(true);
             content_label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
             text_event_box.append(&content_label);
             
             content_vbox.append(&text_event_box);

             // Show More Button (if long text)
             if is_long {
                 let show_more_btn = Button::with_label("Show Content");
                 show_more_btn.add_css_class("flat"); 
                 show_more_btn.set_halign(Align::Start);
                 
                 let full_text = display_content.to_string();
                 let label_clone = content_label.clone();
                 let btn_clone = show_more_btn.clone();
                 
                 show_more_btn.connect_clicked(move |_| {
                     label_clone.set_text(&full_text);
                     label_clone.set_lines(-1); 
                     btn_clone.set_visible(false);
                 });
                 content_vbox.append(&show_more_btn);
             }
             
             content_box.append(&content_vbox);
        }

        // Add click handler to entire hbox (will be added after menu button)

        // Menu Button
        let menu_btn = gtk4::MenuButton::new();
        menu_btn.set_icon_name("view-more-symbolic");
        menu_btn.add_css_class("flat");
        menu_btn.set_valign(Align::Start); // Top align
        menu_btn.set_halign(Align::End);

        let popover = gtk4::Popover::new();
        
        // Track menu state
        let menu_counter_map = menu_counter.clone();
        popover.connect_map(move |_| {
            *menu_counter_map.borrow_mut() += 1;
        });
        let menu_counter_unmap = menu_counter.clone();
        popover.connect_unmap(move |_| {
            let mut c = menu_counter_unmap.borrow_mut();
            if *c > 0 { *c -= 1; }
        });

        let menu_box = gtk4::Box::new(Orientation::Vertical, 5);
        menu_box.set_margin_top(10);
        menu_box.set_margin_bottom(10);
        menu_box.set_margin_start(10);
        menu_box.set_margin_end(10);
        
        // 0. Save Image (Only for Images)
        if item.kind == "image" {
             let save_btn = Button::new();
             let save_lbl = Label::new(Some("Save Image"));
             save_lbl.set_halign(Align::Start);
             save_btn.set_child(Some(&save_lbl));
             save_btn.add_css_class("menu-button");
             
             let item_content_save = item.content.clone();
             let window_save = window.clone();
             let _mc_save = menu_counter.clone();
             let popover_save = popover.clone();
             
             save_btn.connect_clicked(move |_| {
                 popover_save.popdown(); // Close menu instantly to prevent Z-index issues
                 
                 let file_dialog = gtk4::FileDialog::builder()
                     .title("Save Image")
                     .modal(true)
                     .accept_label("Save")
                     .initial_name("image.png")
                     .build();
                 
                 let src_path = std::path::PathBuf::from(&item_content_save);
                 // Try to get original filename
                 if let Some(name) = src_path.file_name() {
                     if let Some(name_str) = name.to_str() {
                         file_dialog.set_initial_name(Some(name_str));
                     }
                 }

                 let window_clone = window_save.clone(); // Clone for async block
                 let src_path_clone = src_path.clone();

                 file_dialog.save(Some(&window_clone), None::<&gtk4::gio::Cancellable>, move |result| {
                     match result {
                         Ok(file) => {
                             if let Some(target_path) = file.path() {
                                 if let Err(e) = std::fs::copy(&src_path_clone, &target_path) {
                                     eprintln!("Failed to save image: {}", e);
                                 } else {
                                     println!("Image saved to {:?}", target_path);
                                 }
                             }
                         }
                         Err(e) => {
                             eprintln!("Save cancelled or failed: {}", e);
                         }
                     }
                 });
             });
             menu_box.append(&save_btn);
             
             let separator = gtk4::Separator::new(Orientation::Horizontal);
             menu_box.append(&separator);
        }

        // 1. Pin/Unpin
        let pin_label = if item.pinned { "Unpin" } else { "Pin" };
        let pin_btn = Button::new();
        let pin_lbl = Label::new(Some(pin_label));
        pin_lbl.set_halign(Align::Start);
        pin_btn.set_child(Some(&pin_lbl));
        pin_btn.add_css_class("menu-button");
        let storage_pin = storage_rc.clone();
        let list_box_pin = list_box.clone();
        let window_pin = window.clone();
        let mc_pin = menu_counter.clone();
        pin_btn.connect_clicked(move |_| {
            {
                let mut s = storage_pin.borrow_mut();
                if let Some(it) = s.history.get_mut(i) {
                    it.pinned = !it.pinned;
                }
                let _ = s.save();
            }
            refresh_list(&list_box_pin, &storage_pin.borrow(), &window_pin, storage_pin.clone(), mc_pin.clone());
        });
        menu_box.append(&pin_btn);

        // 2. Delete
        let delete_btn = Button::new();
        let delete_lbl = Label::new(Some("Delete"));
        delete_lbl.set_halign(Align::Start);
        delete_btn.set_child(Some(&delete_lbl));
        delete_btn.add_css_class("menu-button");
        delete_btn.add_css_class("destructive-action");
        let storage_del = storage_rc.clone();
        let list_box_del = list_box.clone();
        let window_del = window.clone();
        let mc_del = menu_counter.clone();
        delete_btn.connect_clicked(move |_| {
             {
                let mut s = storage_del.borrow_mut();
                if i < s.history.len() {
                    s.history.remove(i);
                }
                 let _ = s.save();
            }
            refresh_list(&list_box_del, &storage_del.borrow(), &window_del, storage_del.clone(), mc_del.clone());
        });
        menu_box.append(&delete_btn);

        popover.set_child(Some(&menu_box));
        menu_btn.set_popover(Some(&popover));


        // Add click gesture to content_box (not hbox) to exclude menu button
        let gesture = gtk4::GestureClick::new();
        let window_clone = window.clone();
        let item_content_for_copy = item_content.clone();
        let item_kind_for_copy = item_kind.clone();
        
        gesture.connect_pressed(move |_, _, _, _| {
            use std::process::{Command, Stdio};
            use std::io::Write;
            
            if item_kind_for_copy == "image" {
                println!("Copying image item");
                
                // Copy Image
                let child = Command::new("wl-copy")
                   .arg("--type").arg("image/png")
                   .stdin(Stdio::from(std::fs::File::open(&item_content_for_copy).expect("Failed to open image file")))
                   .spawn();
                
                match child {
                    Ok(mut child) => {
                        let _ = child.wait();
                    }
                    Err(_) => {
                        // Fallback to xclip
                         let _ = Command::new("xclip")
                           .arg("-selection").arg("clipboard")
                           .arg("-t").arg("image/png")
                           .arg("-i").arg(&item_content_for_copy)
                           .spawn()
                           .and_then(|mut c| c.wait());
                    }
                }
            } else {
                println!("Copying text item");
                
                // Copy Text
                let child = Command::new("wl-copy")
                   .stdin(Stdio::piped())
                   .spawn();
                
                match child {
                    Ok(mut child) => {
                        if let Some(mut stdin) = child.stdin.take() {
                            let _ = stdin.write_all(item_content_for_copy.as_bytes());
                        }
                        let _ = child.wait();
                    }
                    Err(_) => {
                         let child = Command::new("xclip")
                           .arg("-selection")
                           .arg("clipboard")
                           .stdin(Stdio::piped())
                           .spawn();
                         match child {
                             Ok(mut child) => {
                                if let Some(mut stdin) = child.stdin.take() {
                                    let _ = stdin.write_all(item_content_for_copy.as_bytes());
                                }
                                let _ = child.wait();
                             }
                             Err(_) => {}
                         }
                    }
                }
            }
            
            window_clone.close();
        });
        
        content_box.add_controller(gesture);

        hbox.append(&content_box);
        hbox.append(&menu_btn);

        row.set_child(Some(&hbox));
        list_box.append(&row);
    }
}

fn load_css(is_dark: bool) {
    let colors = if is_dark {
        "
        @define-color bg_color #1e1e1e;
        @define-color text_color #ffffff;
        @define-color border_color #333333;
        @define-color hover_bg #252525;
        @define-color hover_text #ffffff;
        @define-color button_text #b0b0b0;
        @define-color destructive #e57373;
        @define-color destructive_hover #ff8a80;
        @define-color pinned #ffd54f;
        @define-color separator #252525;
        @define-color popover_bg #1e1e1e;
        @define-color popover_border #444444;
        "
    } else {
        "
        @define-color bg_color #f5f5f5;
        @define-color text_color #000000;
        @define-color border_color #cccccc;
        @define-color hover_bg #e0e0e0;
        @define-color hover_text #000000;
        @define-color button_text #333333;
        @define-color destructive #d32f2f;
        @define-color destructive_hover #b71c1c;
        @define-color pinned #f57f17;
        @define-color separator #cccccc;
        @define-color popover_bg #ffffff;
        @define-color popover_border #cccccc;
        "
    };

    let base_css = include_str!("style.css");
    // Ensure we don't duplicate if base_css already has it (it shouldn't now)
    let combined_css = format!("{}\n{}", colors, base_css);

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(&combined_css);
    
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

