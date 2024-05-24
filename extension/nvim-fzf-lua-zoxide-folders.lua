--- Dependencies: ibhagwan/fzf-lua
--- Usage:
--- require('nvim-fzf-lua-zoxide-folders').zoxide_folders({ cwd = vim.uv.cwd() })
local M = {}

--- @see https://github.com/ibhagwan/fzf-lua/wiki/Advanced#preview-overview
---@param opts {max_depth?:number,cwd?:string,on_select:function} | table
function M.zoxide_folders(opts)
	if vim.fn.executable("zoxide") == 0 then
		vim.notify("zoxide not installed", vim.log.levels.ERROR)
		return
	end
	local has_git_path_info = vim.fn.executable("path-git-format") == 1
	opts = opts or {}
	opts.formatter = "path.filename_first"

	local fzflua = require("fzf-lua")
	local path = require("fzf-lua.path")

	if not opts.cwd then
		opts.cwd = vim.uv.cwd()
	end
	local preview_cwd = opts.cwd

	local git_branch_info_cmd = ""

	if has_git_path_info then
		git_branch_info_cmd = '| path-git-format --filter --no-bare -f"{path} [{branch}]"'
	end

	-- https://github.com/ibhagwan/fzf-lua/commit/36d850b29b387768e76e59799029d1e69aee2522
	-- opts.fd_opts = string.format('--type directory  --max-depth %s', opts.max_depth or 4)
	-- opts.find_opts = [[-type d -not -path '*/\.git/*' -printf '%P\n']]
	local cmd = string.format(
		[[zoxide query --list --exclude %s %s| awk -v home="$HOME" '{gsub("^" home, "~"); print}']],
		vim.env.HOME,
		git_branch_info_cmd
	)
	local has_exa = vim.fn.executable("eza") == 1

	opts.prompt = "󰥨  Zoxide ❯ "
	opts.cmd = cmd
	opts.cwd_header = true
	opts.cwd_prompt = true
	opts.winopts = {
		fullscreen = false,
		width = 0.7,
		height = 0.5,
	}
	opts.fzf_opts = {
		["--tiebreak"] = "index",
		["--preview-window"] = "nohidden,down,50%",
		["--preview"] = fzflua.shell.raw_preview_action_cmd(function(items)
			local item = (items[1] or ""):gsub("%s%[.*%]$", "")

			if has_exa then
				return string.format(
					"cd %s ; eza --color=always --icons=always --group-directories-first -a %s",
					preview_cwd,
					item
				)
			end
			return string.format("cd %s ; ls %s", preview_cwd, item)
		end),
	}

	opts.actions = {
		["default"] = function(selected, selected_opts)
			local first_selected = selected[1]
			if not first_selected then
				return
			end
			local entry = path.entry_to_file(first_selected, selected_opts)
			local entry_path = entry.path
			if not entry_path then
				return
			end
			entry_path = entry_path:gsub("%s%[.*%]$", "")
			if opts.on_select then
				opts.on_select(entry_path)
			else
				vim.cmd.cd(entry_path)
			end
		end,
	}

	return fzflua.fzf_exec(cmd, opts)
end

return M
