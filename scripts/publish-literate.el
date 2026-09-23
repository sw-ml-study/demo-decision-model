;;; publish-literate.el --- Export a literate document to HTML -*- lexical-binding: t; -*-

;; Run by scripts/publish-literate:
;;   emacs -Q --batch -l scripts/publish-literate.el ORG-FILE MLPL-ELISP-DIR
;;
;; Nothing is evaluated: every #+RESULTS in this repository's documents is
;; recorded by hand and re-run by scripts/check-tangle, so an exporter that
;; executed blocks would be a second, weaker source of truth.
;;
;; sw-MLPL's elisp gives Emacs an MLPL major mode; htmlize turns the faces that
;; mode applies into CSS classes, which each document's own #+HTML_HEAD colours.
;; Without either, the export still succeeds and the blocks are simply plain.

(let* ((args command-line-args-left)
       (org-file (expand-file-name (nth 0 args)))
       (elisp-dir (and (nth 1 args) (expand-file-name (nth 1 args))))
       (all (and elisp-dir (expand-file-name "mlpl-all.el" elisp-dir))))
  (setq command-line-args-left nil)
  (when (and all (file-exists-p all))
    (load all nil t))
  ;; -Q skips package activation, so put an installed htmlize on the load path.
  (dolist (dir (file-expand-wildcards (expand-file-name "~/.emacs.d/elpa/htmlize-*")))
    (add-to-list 'load-path dir))
  (if (require 'htmlize nil t)
      ;; Batch Emacs has no display, so faces carry no colours of their own:
      ;; emit class names and let the document's stylesheet decide.
      (setq org-html-htmlize-output-type 'css
            org-html-htmlize-font-prefix "org-")
    (message "publish-literate: htmlize not found; source blocks will be plain"))
  (require 'org)
  (require 'ox-html)
  (setq org-export-time-stamp-file nil
        org-html-validation-link nil
        org-export-with-broken-links 'mark
        org-confirm-babel-evaluate nil
        make-backup-files nil
        enable-local-variables nil)
  (with-current-buffer (find-file-noselect org-file)
    (org-html-export-to-html)))
