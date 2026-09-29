# Maintainer: Lythium <max@lythium.dev>

pkgname=gmpublisher-bin
_realname=gmpublisher
pkgver=2.12.2
pkgrel=2
pkgdesc="Workshop Publishing Utility for Garry's Mod, written in Rust & Svelte and powered by Tauri"
arch=('x86_64')
url="https://github.com/WilliamVenner/gmpublisher"
license=('GPL-3.0')
depends=('webkit2gtk-4.1' 'libsoup3' 'hicolor-icon-theme' 'libappindicator-gtk3' 'gst-plugins-good' 'gst-plugins-bad' 'gst-libav' 'xdg-desktop-portal')
optdepends=('xdg-desktop-portal-kde: native KDE Plasma file dialogs'
            'kdialog: native KDE Plasma message boxes'
            'zenity: file dialogs when no XDG desktop portal is available')
makedepends=('unzip')
provides=("${_realname}")
conflicts=("${_realname}")
source=("${_realname}_linux64.zip::https://github.com/WilliamVenner/${_realname}/releases/download/${pkgver}/${_realname}_linux64.zip"
        "LICENSE::https://raw.githubusercontent.com/WilliamVenner/${_realname}/${pkgver}/LICENSE")
sha256sums=('SKIP'
            'SKIP')

package() {
  install -Dm755 "${srcdir}/${_realname}" "$pkgdir/usr/lib/${_realname}/${_realname}"
  install -Dm644 "${srcdir}/libsteam_api.so" "$pkgdir/usr/lib/${_realname}/libsteam_api.so"

  install -d "$pkgdir/usr/bin"
  cat << EOF > "$pkgdir/usr/bin/${_realname}"
#!/bin/sh
exec /usr/lib/${_realname}/${_realname} "\$@"
EOF

  chmod 755 "$pkgdir/usr/bin/${_realname}"

  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

  # Launcher, icons, .gma MIME type and Dolphin service menu (see src-tauri/linux)
  cp -r --no-preserve=mode "${srcdir}/share" "$pkgdir/usr/"
}

post_install() {
  /usr/bin/gtk-update-icon-cache -q -t applications -f /usr/share/icons/hicolor
}

post_upgrade() {
  post_install
}

post_remove() {
  /usr/bin/gtk-update-icon-cache -q -t applications -f /usr/share/icons/hicolor
}
