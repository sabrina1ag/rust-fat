# rust-fat
Projet Rust Agrane Sabrina 4SI3
4 actions possibles : 
- **(`ls`)** - Liste les fichiers et répertoires d'un chemin donné
- **(`cat`)** - Lit le contenu d'un fichier à partir d'un chemin absolu ou relatif
- **(`cd`)** - Changer repertoire 
- **(`pwd`)** - Afficher repertoire courant
- **CLI minimaliste** - Interface en ligne de commande pour tester les fonctionnalités

Manquant :  **Créer et écrire dans un fichier** - Non implémenté (nécessite modification de la FAT)

# Contraintes respectes

-  **no_std** - Aucune dépendance sur la bibliothèque standard Rust, **Exception** En dehors du **main.rs** pour avoir une CLI minimale, et les fichiers tests 
-  **alloc** - Utilisation du crate `alloc` pour allocations dynamiques
-  **Tests** - Tests unitaires et d'intégration inclus, il ya 3 erreurs non traité 
-  **Documentation** - Code documenté avec rustdoc
-  **Sécurité** - Toute portion `unsafe` est documentée avec des commentaires de sécurité

# Structure du projet

```
src/
  lib.rs              # Point d'entrée de la bibliothèque
  main.rs             # CLI (nécessite feature "std"), hormis les tests c'est le seul fichier qui est en std
  fs/
    mod.rs            # Module principal du système de fichiers
    boot.rs           # Parcourir le Boot Sector FAT32
    fat_table.rs      # Gestion de la FAT (File Allocation Table)
    fat.rs             # Implémentation principale Fat32Fs
    cluster.rs         # Gestion des chaînes de clusters
    directory.rs       # Gestion des répertoires
    entry.rs           # Entrées de répertoire (short/long names)
    path.rs            # Résolution de chemins (absolu, .. , fichiers, erreur system)
tests/
  integration_fat.rs  # Tests d'intégration
  fat_test.rs
```
# Tester 

**Compiler avec support std pour la CLI**
cargo build --features std --release

**Utiliser la CLI**
cargo run --features std -- <fat32_image>

**ça affiche**
fat32> 
- on pourra alors tester ls, cat, cd, pwd

# Les Images 

# fat32v2.img

<img width="881" height="525" alt="Capture d&#39;écran 2025-12-15 232653" src="https://github.com/user-attachments/assets/26b32279-7a31-43b6-92db-8ed6b7e971b9" />

# fat32.img

Structure de fat32.img (minimale pour les tests)

├── hello.txt
├── subdir/

