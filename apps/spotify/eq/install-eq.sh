#!/bin/sh
# Installe le service d'egaliseur pour l'utilisateur courant (a lancer sur l'appareil, sans root).
set -e
D="$(cd "$(dirname "$0")" && pwd)"
mkdir -p ~/.config/lapp ~/.config/systemd/user
cp "$D/lapp-eq.conf" ~/.config/lapp/lapp-eq.conf
cp "$D/lapp-eq.service" ~/.config/systemd/user/lapp-eq.service
systemctl --user daemon-reload
systemctl --user enable --now lapp-eq.service
