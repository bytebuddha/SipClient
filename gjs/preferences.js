#!/usr/bin/gjs

const Gtk   = imports.gi.Gtk;
const GLib  = imports.gi.GLib;
const { AccountsTable } = imports.widgets.accounts_table;
const { AboutWidget } = imports.widgets.about;
const { ConfigTable } = imports.widgets.config_table;

var PreferencesWindow = function () {
    this.title = 'Nirah Preferences';
    this.window = new Gtk.Window({
      default_height: 300,
      default_width: 620,
      window_position: Gtk.WindowPosition.CENTER
    });
    this.window.set_title(this.title);
    this.label = new Gtk.Label({ label: "..." });
    this.box = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL });
    this.headerBox = new Gtk.Box({orientation: Gtk.Orientation.HORIZONTAL, halign: Gtk.Align.CENTER });
    this.stack = new Gtk.Stack();
    this.stackSwitcher = new Gtk.StackSwitcher({ margin: 20 });
    this.stackSwitcher.stack = this.stack;
    this.headerBox.pack_start(this.stackSwitcher, false, true, 0);
    this.box.pack_start(this.headerBox, false, true, 0);
    this.box.pack_end(this.stack, true, true, 0);
    let accountsTable = new AccountsTable();
    let aboutWidget = new AboutWidget();
    let configTable = new ConfigTable();
    this.stack.add_titled(aboutWidget.widget(), "about", "About");
    this.stack.add_titled(accountsTable.widget(), "accounts", "Accounts");
    this.stack.add_titled(configTable.widget(), "config", "Config");
    this.window.add(this.box);
    this.window.show_all();

};
