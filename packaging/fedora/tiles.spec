Name:           tiles
Version:        0.4.0
Release:        1%{?dist}
Summary:        The classic 2048 puzzle, built with GTK4 and libadwaita

License:        MIT
URL:            https://github.com/hakergeniusz/tiles
Source0:        %{url}/archive/refs/tags/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  gcc
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
install -Dm755 target/release/tiles -t %{buildroot}%{_bindir}/
install -Dm644 packaging/io.github.hakergeniusz.Tiles.desktop -t \
        %{buildroot}%{_datadir}/applications/
install -Dm644 resources/tiles.svg \
        %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.github.hakergeniusz.Tiles.svg
install -Dm644 packaging/io.github.hakergeniusz.Tiles.metainfo.xml -t \
        %{buildroot}%{_metainfodir}/
install -Dm644 LICENSE -t %{buildroot}%{_licensedir}/%{name}/

%files
%{_bindir}/tiles
%{_datadir}/applications/io.github.hakergeniusz.Tiles.desktop
%{_datadir}/icons/hicolor/scalable/apps/io.github.hakergeniusz.Tiles.svg
%{_metainfodir}/io.github.hakergeniusz.Tiles.metainfo.xml
%license LICENSE

%changelog
* Mon Nov 02 2026 hakergeniusz <hakergeniusz@users.noreply.github.com> - 0.4.0-1
- First Tiles release: rename, app icon, deb/rpm/arch/flatpak packaging
