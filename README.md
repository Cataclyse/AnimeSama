# 🍿 Anime-Sama — Netflix Edition (Rust)

Une application desktop moderne et complète écrite en **Rust** offrant une interface inspirée de **Netflix** pour explorer le catalogue, le planning, les épisodes et le streaming directement issus d'Anime-Sama.

---

## 🚀 Fonctionnalités

- **Interface Netflix Authentique** :
  - **Hero Billboard** cinématique avec animes en vedette, défilement automatique, synopsis complet, badges et genres.
  - **Rangées horizontales (Rails Netflix)** avec défilement fluide et flèches de navigation.
  - **Aperçu vidéo au survol** : bandes-annonces lues au survol prolongé avec bouton de contrôle du son.
  - **Fiche détaillée modale** avec synopsis, sélection des saisons, sélection des langues (VOSTFR, VF...) et grille d'épisodes avec miniatures.
  - **Lecteur cinéma intégré** : visionnage direct dans l'application avec contrôles précédent / suivant, choix du lecteur et plein écran.
- **Système Multilingue & Custom Languages** :
  - 10 langues officielles intégrées (.lang) : Français, Anglais, Japonais, Espagnol, Allemand, Italien, Portugais, Arabe, Russe et Coréen.
  - Menu déroulant avec vrais drapeaux SVG vectoriels.
  - **Support des langues personnalisées** : déposez simplement n'importe quel fichier `.lang` dans `%LOCALAPPDATA%\AnimeSama\lang` pour l'ajouter à l'application.
- **Ad Shield Intégré** :
  - Bloqueur de publicités et d'onglets intempestifs sur les lecteurs tiers (VOE, VidMoly, Uqload).
  - Proxy local filtrant les scripts indésirables et les popups trompeuses.
- **Données Centralisées dans `%LOCALAPPDATA%\AnimeSama` (Windows) ou `~/.local/share/AnimeSama` (Linux)** :
  - Vos listes, favoris, historique et sessions sont conservés en lieu sûr dans le dossier utilisateur local (`userdata.json`, `session.json`).
  - Aucun stockage polluant dans le dossier de travail.
- **Synchronisation avec votre compte Anime-Sama** :
  - Connexion optionnelle à votre compte anime-sama.to avec synchronisation automatique ou manuelle de votre progression.

---

## 💻 Démarrage & Exécution

### 🐧 Compatibilité Linux

L'application est **100% compatible Linux** (ainsi que Windows et macOS). Sous Linux, le moteur utilise **GTK 3** et **WebKitGTK** pour la fenêtre native.

#### Dépendances système requises (Linux) :

- **Ubuntu / Debian / Mint** :
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev build-essential
  ```
- **Arch Linux / Manjaro** :
  ```bash
  sudo pacman -S webkit2gtk-4.1 gtk3 base-devel
  ```
- **Fedora / RHEL** :
  ```bash
  sudo dnf install webkit2gtk4.1-devel gtk3-devel
  ```

*Note : si WebKitGTK n'est pas installé ou sur un serveur sans affichage graphique, vous pouvez exécuter `cargo run --release -- --browser` ou `cargo run --release -- --headless`.*

### 1. Lancement Standard (Fenêtre Native Desktop)
```bash
cargo run --release
```
L'application ouvre une fenêtre native (WebView2 sur Windows, WebKitGTK sur Linux avec accélération matérielle et l'icône officielle Anime-Sama).

### 2. Lancement avec Navigateur
Pour ouvrir l'application dans votre navigateur web par défaut :
```bash
cargo run --release -- --browser
```

### 3. Lancement Mode Serveur Seul (Headless)
Pour lancer le serveur local sans ouvrir de fenêtre :
```bash
cargo run --release -- --headless
```
Accédez ensuite à l'interface sur [http://127.0.0.1:4567](http://127.0.0.1:4567).

---

## 🛠 Structure du Projet

- `src/main.rs` : Point d'entrée de l'application, gestion CLI et fenêtre native (`Tao` + `Wry`) avec icône officielle Anime-Sama.
- `src/adblock/` : Moteur de protection Ad Shield contre les popups et traqueurs.
- `src/catalogue/` : Extraction du catalogue temps réel et intégration des trailers vidéo.
- `src/proxy/` : Reverse-proxy local pour contourner les protections iframes et sécuriser les flux de streaming.
- `src/scraper/` : Scraper temps réel d'Anime-Sama (saisons, langues, lecteurs et épisodes).
- `src/server/` : Serveur HTTP multi-thread (`tiny_http`), routes API et distribution des ressources.
- `src/user/` : Gestion du profil, authentification cloud anime-sama.to et gestion des chemins `%LOCALAPPDATA%\AnimeSama`.
- `static/css/` : Feuilles de styles modulaires (thème sombre Netflix, animations, responsive design).
- `static/js/` : Moteur frontend modulaire en JavaScript vanilla sans dépendance externe lourde.
- `static/lang/` : Fichiers de langues `.lang` complets pour l'internationalisation.

---

## 📄 Licence

Ce projet est sous licence **MIT**. Vous êtes libre de l'utiliser, l'étudier, le modifier et le redistribuer à condition de **conserver la notice de copyright originale et les crédits d'auteur** dans toutes les copies ou parties substantielles du logiciel. Consultez le fichier [LICENSE](LICENSE) pour plus de détails.
