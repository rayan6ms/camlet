Name:           camlet-rust
Version:        0.2.4
Release:        1%{?dist}
Summary:        Native floating camera overlay
License:        GPL-3.0-only
URL:            https://github.com/rayan6ms/camlet
Source0:        camlet
Source1:        io.github.rayan6ms.camlet.desktop
Source2:        io.github.rayan6ms.camlet.metainfo.xml
Source3:        camlet-rust.svg
Source4:        camlet-rust-256.png
Source5:        LICENSE
Source6:        README.md
Source7:        camlet.1

Provides:       camlet = %{version}-%{release}

%description
Camlet is a lightweight native floating camera overlay built with Rust, Iced,
and WGPU for presentations, recordings, and calls.

%prep

%build

%install
install -Dm0755 %{SOURCE0} %{buildroot}%{_bindir}/camlet
install -Dm0644 %{SOURCE1} %{buildroot}%{_datadir}/applications/io.github.rayan6ms.camlet.desktop
install -Dm0644 %{SOURCE2} %{buildroot}%{_datadir}/metainfo/io.github.rayan6ms.camlet.metainfo.xml
install -Dm0644 %{SOURCE3} %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.github.rayan6ms.camlet.svg
install -Dm0644 %{SOURCE4} %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/io.github.rayan6ms.camlet.png
install -Dm0644 %{SOURCE5} %{buildroot}%{_licensedir}/%{name}/LICENSE
install -Dm0644 %{SOURCE6} %{buildroot}%{_docdir}/%{name}/README.md
install -Dm0644 %{SOURCE7} %{buildroot}%{_mandir}/man1/camlet.1

%check
test -x %{buildroot}%{_bindir}/camlet
desktop-file-validate %{buildroot}%{_datadir}/applications/io.github.rayan6ms.camlet.desktop
appstreamcli validate --no-net %{buildroot}%{_datadir}/metainfo/io.github.rayan6ms.camlet.metainfo.xml

%files
%license %{_licensedir}/%{name}/LICENSE
%doc %{_docdir}/%{name}/README.md
%{_bindir}/camlet
%{_mandir}/man1/camlet.1*
%{_datadir}/applications/io.github.rayan6ms.camlet.desktop
%{_datadir}/metainfo/io.github.rayan6ms.camlet.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/io.github.rayan6ms.camlet.svg
%{_datadir}/icons/hicolor/256x256/apps/io.github.rayan6ms.camlet.png

%changelog
* Tue Aug 11 2026 Camlet contributors - 0.2.4-1
- Recover from transient camera startup and frame failures.

* Mon Aug 10 2026 Camlet contributors - 0.2.3-1
- Refine menu stacking and sizing, refresh icons, and reduce release size.

* Mon Aug 10 2026 Camlet contributors - 0.2.2-1
- Fix desktop identity, icon alignment, compact menus, and always-on-top behavior.

* Mon Aug 10 2026 Camlet contributors - 0.2.1-1
- Package the native Rust application for RPM-based Linux distributions.
