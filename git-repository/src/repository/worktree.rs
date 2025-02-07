use crate::{worktree, Worktree};

/// Worktree iteration
impl crate::Repository {
    /// Return a list of all _linked_ worktrees sorted by private git dir path as a lightweight proxy.
    ///
    /// Note that these need additional processing to become usable, but provide a first glimpse a typical worktree information.
    pub fn worktrees(&self) -> std::io::Result<Vec<worktree::Proxy<'_>>> {
        let mut res = Vec::new();
        let iter = match std::fs::read_dir(self.common_dir().join("worktrees")) {
            Ok(iter) => iter,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(res),
            Err(err) => return Err(err),
        };
        for entry in iter {
            let entry = entry?;
            let worktree_git_dir = entry.path();
            if worktree_git_dir.join("gitdir").is_file() {
                res.push(worktree::Proxy {
                    parent: self,
                    git_dir: worktree_git_dir,
                })
            }
        }
        res.sort_by(|a, b| a.git_dir.cmp(&b.git_dir));
        Ok(res)
    }

    /// Iterate all _linked_ worktrees in sort order and resolve them, ignoring those without an accessible work tree, into repositories
    /// whose [`worktree()`][crate::Repository::worktree()] is the worktree currently being iterated.
    ///
    /// Note that for convenience all io errors are squelched so if there is a chance for IO errors during
    /// traversal of an owned directory, better use `list()` directly. The latter allows to resolve repositories
    /// even if the worktree checkout isn't accessible.
    pub fn worktree_repos(&self) -> ! {
        todo!()
    }
}

/// Interact with individual worktrees and their information.
impl crate::Repository {
    /// Return the repository owning the main worktree.
    ///
    /// Note that it might be the one that is currently open if this repository dosn't point to a linked worktree.
    /// Also note that the main repo might be bare.
    pub fn main_repo(&self) -> Result<crate::Repository, crate::open::Error> {
        crate::ThreadSafeRepository::open_opts(self.common_dir(), self.options.clone()).map(Into::into)
    }

    /// Return the currently set worktree if there is one, acting as platform providing a validated worktree base path.
    ///
    /// Note that there would be `None` if this repository is `bare` and the parent [`Repository`][crate::Repository] was instantiated without
    /// registered worktree in the current working dir.
    pub fn worktree(&self) -> Option<Worktree<'_>> {
        self.work_dir().map(|path| Worktree { parent: self, path })
    }

    /// Return true if this repository is bare, and has no main work tree.
    ///
    /// This is not to be confused with the [`worktree()`][crate::Repository::worktree()] worktree, which may exists if this instance
    /// was opened in a worktree that was created separately.
    pub fn is_bare(&self) -> bool {
        self.config.is_bare
    }

    /// Open a new copy of the index file and decode it entirely.
    ///
    /// It will use the `index.threads` configuration key to learn how many threads to use.
    /// Note that it may fail if there is no index.
    // TODO: test
    #[cfg(feature = "git-index")]
    pub fn open_index(&self) -> Result<git_index::File, worktree::open_index::Error> {
        use std::convert::{TryFrom, TryInto};
        let thread_limit = self
            .config
            .resolved
            .boolean("index", None, "threads")
            .map(|res| {
                res.map(|value| if value { 0usize } else { 1 }).or_else(|err| {
                    git_config::Integer::try_from(err.input.as_ref())
                        .map_err(|err| worktree::open_index::Error::ConfigIndexThreads {
                            value: err.input.clone(),
                            err,
                        })
                        .map(|value| value.to_decimal().and_then(|v| v.try_into().ok()).unwrap_or(1))
                })
            })
            .transpose()?;
        git_index::File::at(
            self.git_dir().join("index"),
            git_index::decode::Options {
                object_hash: self.object_hash(),
                thread_limit,
                min_extension_block_in_bytes_for_threading: 0,
            },
        )
        .map_err(Into::into)
    }
}
