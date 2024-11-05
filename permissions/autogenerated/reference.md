| Permission | Description |
|------|-----|
|`allow-available-ports`|Enables the available_ports command without any pre-configured scope.|
|`deny-available-ports`|Denies the available_ports command without any pre-configured scope.|
|`allow-cancel-read`|Enables the cancel_read command without any pre-configured scope.|
|`deny-cancel-read`|Denies the cancel_read command without any pre-configured scope.|
|`allow-close`|Enables the close command without any pre-configured scope.|
|`deny-close`|Denies the close command without any pre-configured scope.|
|`allow-close-all`|Enables the close_all command without any pre-configured scope.|
|`deny-close-all`|Denies the close_all command without any pre-configured scope.|
|`allow-force-close`|Enables the force_close command without any pre-configured scope.|
|`deny-force-close`|Denies the force_close command without any pre-configured scope.|
|`allow-open`|Enables the open command without any pre-configured scope.|
|`deny-open`|Denies the open command without any pre-configured scope.|
|`allow-read`|Enables the read command without any pre-configured scope.|
|`deny-read`|Denies the read command without any pre-configured scope.|
|`allow-write`|Enables the write command without any pre-configured scope.|
|`deny-write`|Denies the write command without any pre-configured scope.|
|`allow-write-binary`|Enables the write_binary command without any pre-configured scope.|
|`deny-write-binary`|Denies the write_binary command without any pre-configured scope.|
|`default`|# Tauri `fs` default permissions

This configuration file defines the default permissions granted
to the filesystem.

### Granted Permissions

This default permission set enables all read-related commands and
allows access to the `$APP` folder and sub directories created in it.
The location of the `$APP` folder depends on the operating system,
where the application is run.

In general the `$APP` folder needs to be manually created
by the application at runtime, before accessing files or folders
in it is possible.

### Denied Permissions

This default permission set prevents access to critical components
of the Tauri application by default.
On Windows the webview data folder access is denied.

|
|`read`|This enables all read related commands|
|`write`|This enables all write related commands|
