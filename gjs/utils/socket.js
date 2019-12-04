'use-strict';

const Gio = imports.gi.Gio;

var NirahSocket = class nirahSocket {
  constructor() {
    this._addr = new Gio.UnixSocketAddress({ path: "/tmp/nirah/nirah-bytebuddha.socket"});
    this._sock = new Gio.SocketClient();
    this._sock.set_family(1);
  }

  connect() {
    try {
      this._conn = this._sock.connect(this._addr, null);
      return true;
    } catch(_err) {
      log(_err);
      return false;
    }
  }

  send_message(msg) {
    if(this.connect()) {
      let data;
      try {
        data = JSON.stringify(msg);
        this._conn.get_output_stream()
        .write_all(data+'\n', null);
      } catch(err) {
        log("Failed to send rpc message: "+err.toString());
      }
    } else {
        log("Failed to connect to nirah socket.");
    }
  }

  read_message() {
    let output_reader = Gio.DataInputStream.new(this._conn.get_input_stream());
    let [output, count] = output_reader.read_line(null);
    print(JSON.stringify(imports.byteArray.toString(output)));
    return JSON.parse(imports.byteArray.toString(output));
  }

  send_then(msg, func) {
    this.send_message(msg);
    let res = this.read_message();
    if(res.response == 'Ok') {
      func(res);
    } else {
      print(JSON.stringify(res));
    }
  }

  send_then_expect(msg, expected, func) {
    this.send_message(msg);
    let res = this.read_message();
    if(res.response == expected) {
      func(res);
    } else {
      print(JSON.stringify(res));
    }
  }

  editAccountUsernameRequest(account, username) {
    return {
      method: 'EditAccountUsername',
      account,
      username
    };
  }

  editAccountPasswordRequest(account, password) {
    return {
      method: 'EditAccountPassword',
      account,
      password
    };
  }

  editAccountHostnameRequest(account, host) {
    return {
      method: 'EditAccountHost',
      account,
      host
    };
  }
};
