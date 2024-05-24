function jump-first --description="Zoxide jump(first) with git branch in path"
    set -l query "$argv"
    set result (zoxide query --list --exclude $PWD | path-git-format --filter --no-bare -f"{path} [{branch}]" | fzf --tiebreak=index --exact --filter="$query" --no-sort --nth=4.. --delimiter='[\/\s]' | head -n 1)
    if test -n "$result"
        set directory (echo $result | awk -F' ' '{print $1}' | awk -F'[' '{print $1}')
        if test "$_ZO_ECHO" = 1
            echo "$directory"
        end
        eval cd $directory
    end
end

function jump --description="Zoxide jump with git branch in path"
    set -l query "$argv"
    set result (zoxide query --list --exclude $PWD | path-git-format --filter --no-bare -f"{path} [{branch}]" | awk -v home="$HOME" '{gsub(home, "~", $1); print $0}' | fzf --height ~60% --reverse --tiebreak=index -1 -0 --exact --query="$query")
    if test -n "$result"
        set directory (echo $result | awk -F' ' '{print $1}' | awk -F'[' '{print $1}')
        if test "$_ZO_ECHO" = 1
            echo "$directory"
        end
        eval cd $directory
    end
end

