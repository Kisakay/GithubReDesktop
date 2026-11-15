app-title = GitHub ReDesktop
settings = Paramètres

## About
about-repository = Code source
about-issues = Signaler un problème
about-original = GitHub Desktop original
about-comments = Une réécriture native de GitHub Desktop pour Linux, en Rust avec libcosmic.

## Tabs
tab-changes = Modifications
tab-history = Historique

## Menus
menu-file = Fichier
menu-view = Affichage
menu-repository = Dépôt
menu-branch = Branche
menu-help = Aide
menu-new-repository = Nouveau dépôt…
menu-add-local = Ajouter un dépôt local…
menu-clone = Cloner un dépôt…
menu-quit = Quitter
menu-push = Pousser (push)
menu-pull = Tirer (pull)
menu-fetch = Récupérer (fetch)
menu-force-push = Forcer le push…
menu-open-editor = Ouvrir dans l'éditeur
menu-open-terminal = Ouvrir dans le terminal
menu-show-in-files = Afficher dans le gestionnaire de fichiers
menu-view-on-github = Voir sur GitHub
menu-refresh = Actualiser
menu-remove-repository = Retirer le dépôt…
menu-new-branch = Nouvelle branche…
menu-rename-branch = Renommer la branche…
menu-delete-branch = Supprimer la branche…
menu-merge = Fusionner dans la branche courante…
menu-compare-on-github = Comparer sur GitHub
menu-stash = Remiser toutes les modifications
menu-stash-pop = Restaurer les modifications remisées
menu-discard-all = Annuler toutes les modifications…
menu-report-issue = Signaler un problème…
menu-about = À propos de GitHub ReDesktop
no-repository-selected = Aucun dépôt sélectionné

## Toolbar
current-branch = Branche actuelle
detached-head = Détachée sur { $sha }
fetch-origin = Fetch { $remote }
pull-origin = Pull { $remote }
push-origin = Push { $remote }
publish-repository = Publier le dépôt
publish-repository-caption = Publier ce dépôt sur GitHub
publish-branch = Publier la branche
publish-branch-caption = Publier cette branche sur le distant
last-fetched = Dernier fetch { $when }
never-fetched = Jamais récupéré
please-wait = Veuillez patienter…
restore-stash = Restaurer la remise ({ $count })

## Operations
op-fetching = Récupération…
op-pulling = Pull en cours…
op-pushing = Push en cours…
op-publishing = Publication…
op-cloning = Clonage…
op-working = En cours…
done-fetch = Récupéré
done-pull = Dernières modifications récupérées
done-push = Poussé sur le distant
done-publish = Publié sur GitHub
done-commit = Modifications commitées
done-merge = Fusion terminée
done-stash = Modifications remisées
done-stash-pop = Modifications remisées restaurées
done-revert = Commit annulé (revert)
done-cherry-pick = Commit appliqué (cherry-pick)
done-undo = Dernier commit défait
done-delete-branch = Branche supprimée
stash-message = Remise GitHub ReDesktop sur { $branch }

## Merge
merge-in-progress = Une fusion est en cours.
merge-conflicts = { $count ->
    [one] Un fichier est en conflit. Résolvez-le dans votre éditeur, puis commitez.
   *[other] { $count } fichiers sont en conflit. Résolvez-les dans votre éditeur, puis commitez.
}
abort-merge = Abandonner la fusion
merge-title = Fusionner dans { $branch }
merge-into = Fusionner dans { $branch }
merge-commit = Commit de fusion

## Branch popup
filter-branches = Filtrer les branches
local-branches = Branches
remote-branches = Branches distantes
no-branches = Aucune branche ne correspond
current = actuelle
remote = distante

## Changes
changed-files = { $count ->
    [0] Aucun fichier modifié
    [one] 1 fichier modifié
   *[other] { $count } fichiers modifiés
}
no-local-changes = Aucune modification locale
no-local-changes-body = Il n'y a aucune modification non commitée dans { $repo }. Voici quelques suggestions pour la suite.
summary-placeholder = Résumé (obligatoire)
description-placeholder = Description
amend-last-commit = Modifier le dernier commit (amend)
amend-commit = Modifier le dernier commit
commit-to = Commiter sur { $branch }
commit-files-to = { $count ->
    [one] Commiter 1 fichier sur { $branch }
   *[other] Commiter { $count } fichiers sur { $branch }
}
discard-changes = Annuler les modifications…
ignore-file = Ignorer le fichier (ajouter au .gitignore)
ignore-extension = Ignorer tous les fichiers avec cette extension
copy-path = Copier le chemin du fichier
open-file = Ouvrir avec le programme par défaut
loading = Chargement…
loading-diff = Chargement du diff…
binary-file = Fichier binaire
binary-file-body = Ce fichier binaire a changé et ne peut pas être affiché comme du texte.
no-diff = Aucune modification de contenu
no-diff-body = Seuls le mode ou le nom du fichier ont changé.
diff-truncated = Ce diff est très volumineux : seul le début est affiché.

## Suggestions
suggest-publish-body = Publiez ce dépôt sur GitHub pour le partager et le sauvegarder.
suggest-publish-branch-body = Cette branche n'existe que sur votre ordinateur. Publiez-la pour la partager.
suggest-push-title = { $count ->
    [one] 1 commit local attend d'être poussé
   *[other] { $count } commits locaux attendent d'être poussés
}
suggest-push-body = Poussez vos commits pour les partager avec vos collaborateurs.
suggest-pull-title = { $count ->
    [one] 1 nouveau commit sur le distant
   *[other] { $count } nouveaux commits sur le distant
}
suggest-pull-body = Faites un pull pour intégrer ces modifications à votre branche.
suggest-editor-title = Ouvrir le dépôt dans votre éditeur
suggest-editor-body = Modifiez votre code avec votre éditeur préféré.
suggest-files-title = Voir les fichiers du dépôt
suggest-files-body = Parcourez le dépôt dans votre gestionnaire de fichiers.
suggest-github-title = Ouvrir la page du dépôt sur GitHub
suggest-github-body = Consultez les issues, pull requests et plus dans votre navigateur.

## History
no-commits = Aucun commit pour l'instant
load-more = Charger plus de commits
select-commit = Aucun commit sélectionné
select-commit-body = Sélectionnez un commit dans la liste pour voir ses modifications.
files-changed = { $count ->
    [one] 1 fichier modifié
   *[other] { $count } fichiers modifiés
}
undo-commit = Défaire le commit
revert-commit = Annuler les modifications du commit (revert)
cherry-pick = Appliquer sur la branche courante (cherry-pick)
branch-from-commit = Créer une branche depuis ce commit…
copy-sha = Copier le SHA
copy-summary = Copier le résumé

## Dialogs
cancel = Annuler
choose = Choisir…
name = Nom
description = Description
optional = Facultatif
local-path = Chemin local
clone = Cloner
clone-title = Cloner un dépôt
clone-url = URL du dépôt ou utilisateur/dépôt GitHub
clone-url-placeholder = https://github.com/owner/repo.git ou owner/repo
clone-sign-in-hint = Connectez-vous à GitHub dans les paramètres pour choisir un de vos dépôts dans une liste.
filter-repositories = Filtrer vos dépôts
fork = fork
create-title = Créer un nouveau dépôt
create-repository = Créer le dépôt
repo-name-placeholder = mon-super-projet
will-be-created = Le dépôt sera créé dans { $path }
gitignore = .gitignore
gitignore-placeholder = ex. target/  node_modules/  *.log
init-readme = Initialiser ce dépôt avec un README
branch-name-placeholder = ma-nouvelle-fonctionnalite
branch-based-on = Votre nouvelle branche sera basée sur { $branch }.
branch-based-on-commit = Votre nouvelle branche sera basée sur le commit { $sha }.
will-be-created-as = Sera créée sous le nom { $name }
create-branch = Créer la branche
rename-branch-title = Renommer { $branch }
rename = Renommer
keep-private = Garder ce code privé
publish-to = Le dépôt sera créé sur le compte { $account }.
discard-title = { $count ->
    [one] Annuler les modifications d'1 fichier ?
   *[other] Annuler les modifications de { $count } fichiers ?
}
discard-body = Les modifications seront perdues. Les fichiers non suivis sont mis à la corbeille si possible.
and-more = …et { $count } de plus
delete-branch-title = Supprimer la branche
delete-branch-body = Supprimer la branche locale « { $branch } » ? Cette action est irréversible.
delete = Supprimer
remove-repo-title = Retirer le dépôt
remove-repo-body = Retirer { $path } de la liste ? Les fichiers sur le disque ne seront pas supprimés.
remove = Retirer
force-push-title = Forcer le push ?
force-push-body = Forcer le push réécrit l'historique sur le distant. Toute personne travaillant sur cette branche devra réinitialiser sa copie locale.

## Account & settings
account = Compte GitHub
sign-in = Se connecter
sign-out = Se déconnecter
signed-in-as = Connecté en tant que { $login }
sign-in-body = Collez un jeton d'accès personnel (portées : repo, read:org, workflow) ou réutilisez votre connexion GitHub CLI. Le jeton est stocké dans le trousseau du système.
token-placeholder = ghp_…
use-gh-cli = Utiliser la connexion GitHub CLI
create-token = Créer un jeton
gh-not-found = Impossible d'obtenir un jeton depuis GitHub CLI. Lancez d'abord « gh auth login ».
sign-in-to-publish = Connectez-vous à GitHub pour publier ce dépôt.
git-identity = Identité Git
git-name = Nom
git-name-placeholder = Votre nom
git-email = E-mail
save = Enregistrer
set-identity = Configurer votre nom et e-mail Git…
identity-saved = Identité Git enregistrée
integrations = Intégrations
editor = Éditeur externe
terminal = Terminal
auto-detect = Détection automatique
detected = Détectés : { $list }
none-detected = aucun
clone-dir = Dossier de clonage par défaut
appearance-behavior = Apparence et comportement
theme = Thème
theme-system = Suivre le bureau
theme-dark = Sombre
theme-light = Clair
auto-fetch = Récupérer en arrière-plan toutes les 5 minutes
confirm-discard = Confirmer avant d'annuler des modifications

## Welcome
welcome-title = C'est parti !
welcome-body = Ajoutez un dépôt à GitHub ReDesktop pour commencer à collaborer.
welcome-clone = Cloner un dépôt depuis Internet…
welcome-clone-body = Depuis GitHub.com ou n'importe quelle URL Git
welcome-create = Créer un nouveau dépôt sur votre ordinateur…
welcome-create-body = Démarrer un nouveau projet avec Git
welcome-add = Ajouter un dépôt existant de votre ordinateur…
welcome-add-body = Choisissez un dossier qui contient déjà un dépôt Git
welcome-sign-in = Connectez-vous à GitHub pour voir vos dépôts

## Errors
add-local-title = Choisir un dépôt local
not-a-repository = { $path } n'est pas un dépôt Git.
already-a-repository = Ce dossier est déjà un dépôt Git.
destination-not-empty = La destination { $path } existe déjà et n'est pas vide.
repository-missing = Le dépôt { $path } est introuvable. Il a peut-être été déplacé ou supprimé.
no-editor-found = Aucun éditeur trouvé. Configurez-en un dans les paramètres.
no-terminal-found = Aucun terminal trouvé. Configurez-en un dans les paramètres.

## Time
time-just-now = à l'instant
time-minutes = { $count ->
    [one] il y a 1 minute
   *[other] il y a { $count } minutes
}
time-hours = { $count ->
    [one] il y a 1 heure
   *[other] il y a { $count } heures
}
time-days = { $count ->
    [one] il y a 1 jour
   *[other] il y a { $count } jours
}
time-months = { $count ->
    [one] il y a 1 mois
   *[other] il y a { $count } mois
}
time-years = { $count ->
    [one] il y a 1 an
   *[other] il y a { $count } ans
}
