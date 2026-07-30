# Maintainer: Parch Linux Team
pkgname=pordle
pkgver=0.1.0
pkgrel=1
pkgdesc="A Persian Wordle game for Parch Linux"
arch=('x86_64')
url="https://github.com/parchlinux/pordle"
license=('AGPL3')
depends=('gtk4' 'libadwaita' 'hicolor-icon-theme')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/parchlinux/pordle/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "$srcdir/$pkgname-$pkgver"
    export CARGO_TARGET_DIR="$srcdir/pordle-target"
    cargo build --release --frozen
}

package() {
    cd "$srcdir/$pkgname-$pkgver"

    install -Dm755 "$CARGO_TARGET_DIR/release/pordle" "$pkgdir/usr/bin/pordle"

    install -Dm644 data/com.parchlinux.pordle.desktop \
        "$pkgdir/usr/share/applications/com.parchlinux.pordle.desktop"

    install -Dm644 data/icons/com.parchlinux.pordle.svg \
        "$pkgdir/usr/share/icons/hicolor/scalable/apps/com.parchlinux.pordle.svg"
}
