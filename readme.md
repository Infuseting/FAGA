<div align="center">
  <h1>FAGA Browser</h1>
  <p>
    <strong>Free. Anonymous. Guarded. Access.</strong><br>
    Le navigateur web souverain, forgé entièrement en Rust.
  </p>
  <p>
    <a href="https://github.com/Infuseting/FAGA/actions"><img src="https://img.shields.io/badge/build-experimental-orange?style=flat-square" alt="Build Status" /></a>
    <a href="https://crates.io/"><img src="https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square&logo=rust" alt="Rust Version" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License" /></a>
    <a href="#"><img src="https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey?style=flat-square" alt="Platform" /></a>
    <a href="#"><img src="https://img.shields.io/badge/privacy-extreme-red?style=flat-square" alt="Privacy Focused" /></a>
  </p>

  <h4>
    <a href="#-à-propos">À propos</a> •
    <a href="#-fonctionnalités-clés">Fonctionnalités</a> •
    <a href="#-installation">Installation</a> •
    <a href="#-architecture">Architecture</a> •
    <a href="#-roadmap">Roadmap</a>
  </h4>
</div>

---

## 📖 À propos

**FAGA** (Free Anonymous Guarded Access) est un projet ambitieux visant à implémenter un navigateur moderne écrit en **Rust**, depuis les parsers HTML/CSS jusqu'à l'UI native (via `iced`). L'objectif est d'expérimenter un moteur léger, sûr et respectueux de la vie privée.

> ⚠️ État : Alpha / expérimental — beaucoup de composants sont en prototype. Ne l'utilisez pas pour des besoins critiques.

---

## ⚡ Fonctionnalités clés (implémentées dans cette branche)

- UI native multi-OS (Windows / macOS / Linux) avec barre d'onglets, barre d'adresse, contrôles de fenêtre personnalisés.
- Parser HTML/CSS minimal fonctionnel et renderer qui applique :
  - Unités : px, em, rem, %, vw, vh, pt.
  - Marges, paddings, width (y compris `width:60vw`) et `margin: 15vh auto` (centrage horizontal lorsque la largeur est définie).
  - Styles inline et feuille de style par défaut (`assets/css/default.css`).
- Résolution des `em` et `rem` relative au parent / root. Correction du calcul d'unités relatives.
- Liens `<a href>` cliquables ; résolution d'URLs relatives vers absolues (`resolve_url`).
- DevTools simplifié (F12 / Ctrl+Shift+I) : panels Elements / Styles / Console / Network pour inspection et logs.
- Support des touches et événements fenêtre (redimensionnement, drag pour déplacer la fenêtre, contrôles min/max/close).
- Logging via `env_logger` (activer avec `RUST_LOG`).

---

## 🏗️ Architecture (aperçu)

Le projet est organisé en modules :

- `src/main.rs` : UI (iced), gestion des onglets, rendu final et DevTools.
- `src/parser/` : parsers HTML, CSS, DOM et `HtmlRenderer` (calcule `ComputedStyles` et génère un `RenderNode`).
- `src/network/` : client HTTP minimal utilisé pour charger les pages.
- `assets/css/default.css` : CSS par défaut chargé pour émuler un style de navigateur.

Diagramme (pipeline simplifié) :

```
Network -> HTML bytes -> HTML Parser -> DOM
         CSS bytes  -> CSS Parser  -> CSSOM
DOM + CSSOM -> Computed Styles -> Render tree -> UI (iced)
```

---

## 🛠️ Installation & build

### Prérequis
- Rust (stable) — Rust 1.75+ recommandé.
- Outils système (Linux) : `pkg-config`, `libssl-dev`, `libfontconfig`, `libfreetype` si nécessaire.

### Build & run

Sous PowerShell (Windows) :

```powershell
cd C:\Users\Arthur\RustroverProjects\FAGA
# debug
cargo run
# release
cargo build --release
```

Sous bash (Linux / macOS) :

```bash
cd /chemin/vers/FAGA
cargo run
# ou
cargo build --release
```

Logs utiles :

```bash
# niveau info
RUST_LOG=info cargo run
# niveau debug (plus verbeux)
RUST_LOG=debug cargo run
```

---

## 🧭 Raccourcis et interactions

- F12 ou Ctrl+Shift+I : ouvrir/fermer DevTools
- Cliquer un lien `<a>` : navigation (résolution relative automatique)
- Cliquer-glisser un onglet : réordonner
- Glisser la barre d'onglets : déplacer la fenêtre
- Boutons personnalisés en haut à droite : minimiser / maximiser / fermer

---

## 🔎 Tests rapides / Vérifications

- Tester `width:60vw;margin:15vh auto` :
  - Ouvrir une page contenant `<style>body{width:60vw;margin:15vh auto}</style>`
  - Redimensionner la fenêtre : la largeur du contenu doit suivre (~60% de la largeur réelle) et la marge haut être ~15% de la hauteur.
- Inspecter un `h1` via DevTools pour vérifier le calcul `em` (devtools loge les tailles avant/après).
- Cliquer sur les liens pour vérifier la navigation et la résolution d'URL.

---

## ⚠️ Limitations connues

- Parser HTML/CSS basique : pas de cascade complète, pas de layout CSS avancé (flex/grid/positioning complexe).
- Pas (encore) de support JavaScript.
- Multi-fenêtre / détachement d'onglet vers nouvelle fenêtre : pas implémenté (TODO : iced multi-window).
- Moteur réseau minimal : fonctionnalités limités (cookies/redirects/HTTP2/SSL edge cases).
- Accessibilité : tailles de cibles pensées pour l'accessibilité mais tests complémentaires nécessaires.

---

## 🛣️ Roadmap (prochaine priorité)

1. Stabiliser le rendu des unités (em/rem/vw/vh) → DONE (implémenté pour la branche courante)
2. Ajouter plus de tests unitaires pour `resolve_url`, parser CSS (vw/vh/em)
3. Améliorer le layout : support des blocs inline/flow, marges collapse, boîtes
4. Explorer une option JS (intégration d'un moteur JS sandboxé) — gros chantier
5. Multi-fenêtre et drag & drop inter-fenêtres
6. Polices fallback / meilleur support d'encodages exotiques

---

## 🤝 Contribution
Contributions bienvenues : fork → branche → PR. Merci d'ajouter des tests et de documenter les changements majeurs.

---

## 📄 Licence
Ce projet est sous licence **MIT** — voir le fichier `LICENSE`.

---

Si vous voulez que j'ajoute :
- des pages HTML de test sous `assets/tests/` ;
- des tests unitaires pour `resolve_url` ;
- une checklist détaillée pour le renderer CSS ;

dites-moi quelles options vous préférez et je les ajoute.
