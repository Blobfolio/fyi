_basher__fyi_aborted() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_blank() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -c " ]] && [[ ! " ${COMP_LINE} " =~ " --count " ]]; then
		opts+=("-c")
		opts+=("--count")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_confirm() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -y " ]] && [[ ! " ${COMP_LINE} " =~ " --yes " ]]; then
		opts+=("-y")
		opts+=("--yes")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_crunched() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_debug() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_done() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_error() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_found() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_info() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_notice() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_print() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -p " ]] && [[ ! " ${COMP_LINE} " =~ " --prefix " ]]; then
		opts+=("-p")
		opts+=("--prefix")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -c " ]] && [[ ! " ${COMP_LINE} " =~ " --prefix-color " ]]; then
		opts+=("-c")
		opts+=("--prefix-color")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_review() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_skipped() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_success() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_task() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher__fyi_warning() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -i " ]] && [[ ! " ${COMP_LINE} " =~ " --indent " ]]; then
		opts+=("-i")
		opts+=("--indent")
	fi
	[[ " ${COMP_LINE} " =~ " --stderr " ]] || opts+=("--stderr")
	if [[ ! " ${COMP_LINE} " =~ " -t " ]] && [[ ! " ${COMP_LINE} " =~ " --timestamp " ]]; then
		opts+=("-t")
		opts+=("--timestamp")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -e " ]] && [[ ! " ${COMP_LINE} " =~ " --exit " ]]; then
		opts+=("-e")
		opts+=("--exit")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
_basher___fyi() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -V " ]] && [[ ! " ${COMP_LINE} " =~ " --version " ]]; then
		opts+=("-V")
		opts+=("--version")
	fi
	opts+=("aborted")
	opts+=("blank")
	opts+=("confirm")
	opts+=("crunched")
	opts+=("debug")
	opts+=("done")
	opts+=("error")
	opts+=("found")
	opts+=("info")
	opts+=("notice")
	opts+=("print")
	opts+=("review")
	opts+=("skipped")
	opts+=("success")
	opts+=("task")
	opts+=("warning")
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
subcmd__basher___fyi() {
	local i cmd
	COMPREPLY=()
	cmd=""
	for i in ${COMP_WORDS[@]}; do
		case "${i}" in
			fyi)
				cmd="fyi"
				;;
			aborted)
				cmd="aborted"
				;;
			blank)
				cmd="blank"
				;;
			confirm)
				cmd="confirm"
				;;
			crunched)
				cmd="crunched"
				;;
			debug)
				cmd="debug"
				;;
			done)
				cmd="done"
				;;
			error)
				cmd="error"
				;;
			found)
				cmd="found"
				;;
			info)
				cmd="info"
				;;
			notice)
				cmd="notice"
				;;
			print)
				cmd="print"
				;;
			review)
				cmd="review"
				;;
			skipped)
				cmd="skipped"
				;;
			success)
				cmd="success"
				;;
			task)
				cmd="task"
				;;
			warning)
				cmd="warning"
				;;
			*)
				;;
		esac
	done
	echo "$cmd"
}
chooser__basher___fyi() {
	local i cmd
	COMPREPLY=()
	cmd="$( subcmd__basher___fyi )"
	case "${cmd}" in
		fyi)
			_basher___fyi
			;;
		aborted)
			_basher__fyi_aborted
			;;
		blank)
			_basher__fyi_blank
			;;
		confirm)
			_basher__fyi_confirm
			;;
		crunched)
			_basher__fyi_crunched
			;;
		debug)
			_basher__fyi_debug
			;;
		done)
			_basher__fyi_done
			;;
		error)
			_basher__fyi_error
			;;
		found)
			_basher__fyi_found
			;;
		info)
			_basher__fyi_info
			;;
		notice)
			_basher__fyi_notice
			;;
		print)
			_basher__fyi_print
			;;
		review)
			_basher__fyi_review
			;;
		skipped)
			_basher__fyi_skipped
			;;
		success)
			_basher__fyi_success
			;;
		task)
			_basher__fyi_task
			;;
		warning)
			_basher__fyi_warning
			;;
		*)
			;;
	esac
}
complete -F chooser__basher___fyi -o bashdefault -o default fyi
