# Flatpak evaluation

A Flatpak manifest is deferred until SonicFetch has a graphical application and stable release identity. The current binary is a foreground CLI, so a desktop launcher, portal-based destination chooser, and desktop integration would be misleading.

When the GTK application is available, evaluate:

- GNOME runtime and GTK/libadwaita version compatibility.
- Network permission and user-selected file access through portals.
- SQLite data location under the Flatpak app data directory.
- Offline build reproducibility and Cargo dependency vendoring.
- AppStream metadata, desktop entry, icon, and release signing.

Do not grant broad host filesystem access by default. Downloads should use user-selected locations via document portal access where possible.
