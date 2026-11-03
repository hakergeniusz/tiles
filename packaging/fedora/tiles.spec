Name:           tiles
Version:        51.0.0
Release:        0.1.beta.1%{?dist}
Summary:        The classic 2048 puzzle, built with GTK4 and libadwaita

License:        MIT
URL:            https://github.com/hakergeniusz/tiles
Source0:        %{url}/archive/refs/tags/v51.0.0-beta.1/%{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  glib2-devel
Requires:       gtk4
Requires:       libadwaita

%description
Tiles is the classic 2048 puzzle as a native Linux desktop app, built with
Rust, GTK4 and libadwaita: animated moves, Light/Sepia/Dark color modes,
mouse/touch swipes and a persisted best score.

%prep
%autosetup

%build
cargo build --release

%install
install -Dm755 target/release/tiles %{buildroot}%{_bindir}/tiles
install -Dm644 packaging/io.github.hakergeniusz.Tiles.desktop \
        %{buildroot}%{_datadir}/applications/io.github.hakergeniusz.Tiles.desktop
install -Dm644 resources/tiles.svg \
        %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.github.hakergeniusz.Tiles.svg
install -Dm644 packaging/io.github.hakergeniusz.Tiles.metainfo.xml \
        %{buildroot}%{_metainfodir}/io.github.hakergeniusz.Tiles.metainfo.xml
install -Dm644 LICENSE %{buildroot}%{_licensedir}/%{name}/LICENSE

%files
%{_bindir}/tiles
%{_datadir}/applications/io.github.hakergeniusz.Tiles.desktop
%{_datadir}/icons/hicolor/scalable/apps/io.github.hakergeniusz.Tiles.svg
%{_metainfodir}/io.github.hakergeniusz.Tiles.metainfo.xml
%license LICENSE

%changelog
* Tue Nov 03 2026 hakergeniusz <hakergeniusz@users.noreply.github.com> - 51.0.0-0.1.beta.1
- First beta of the 51 series (pre-release of 51.0.0)

* Mon Nov 02 2026 hakergeniusz <hakergeniusz@users.noreply.github.com> - 0.4.0-1
- First Tiles release: rename, app icon, deb/rpm/arch/flatpak packaging
