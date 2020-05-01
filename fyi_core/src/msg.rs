/*!
# FYI Core: Msg
*/

use bytes::{
	BytesMut,
	BufMut
};
use crate::{
	MSG_TIMESTAMP,
	PRINT_NEWLINE,
	PRINT_STDERR,
	traits::{
		AnsiBitsy,
		Elapsed,
		Inflection,
		MebiSaved,
	},
	util::{
		cli,
		strings,
	},
};
use num_format::{
	Locale,
	ToFormattedString,
};
use std::{
	borrow::Cow,
	fmt,
	ops::Deref,
	time::Instant,
};



#[derive(Debug, Clone, Copy, PartialEq)]
/// Custom ANSI color helper.
///
/// BASH inconveniently numbers 1-256 instead of 0-255, so this enum
/// technically loses the ability to render true white, but fuck it.
pub enum Color {
	/// ANSI color in normal weight.
	Normal(u8),
	/// ANSI color bolded.
	Bold(u8),
	/// ANSI color dimmed.
	Dim(u8),
}

impl Deref for Color {
	type Target = str;

	#[allow(clippy::too_many_lines)]
	/// Deref.
	fn deref(&self) -> &Self::Target {
		match *self {
			Self::Normal(c) => match c {
				0 => "",
				1 => "\x1B[38;5;1m",
				2 => "\x1B[38;5;2m",
				3 => "\x1B[38;5;3m",
				4 => "\x1B[38;5;4m",
				5 => "\x1B[38;5;5m",
				6 => "\x1B[38;5;6m",
				7 => "\x1B[38;5;7m",
				8 => "\x1B[38;5;8m",
				9 => "\x1B[38;5;9m",
				10 => "\x1B[38;5;10m",
				11 => "\x1B[38;5;11m",
				12 => "\x1B[38;5;12m",
				13 => "\x1B[38;5;13m",
				14 => "\x1B[38;5;14m",
				15 => "\x1B[38;5;15m",
				16 => "\x1B[38;5;16m",
				17 => "\x1B[38;5;17m",
				18 => "\x1B[38;5;18m",
				19 => "\x1B[38;5;19m",
				20 => "\x1B[38;5;20m",
				21 => "\x1B[38;5;21m",
				22 => "\x1B[38;5;22m",
				23 => "\x1B[38;5;23m",
				24 => "\x1B[38;5;24m",
				25 => "\x1B[38;5;25m",
				26 => "\x1B[38;5;26m",
				27 => "\x1B[38;5;27m",
				28 => "\x1B[38;5;28m",
				29 => "\x1B[38;5;29m",
				30 => "\x1B[38;5;30m",
				31 => "\x1B[38;5;31m",
				32 => "\x1B[38;5;32m",
				33 => "\x1B[38;5;33m",
				34 => "\x1B[38;5;34m",
				35 => "\x1B[38;5;35m",
				36 => "\x1B[38;5;36m",
				37 => "\x1B[38;5;37m",
				38 => "\x1B[38;5;38m",
				39 => "\x1B[38;5;39m",
				40 => "\x1B[38;5;40m",
				41 => "\x1B[38;5;41m",
				42 => "\x1B[38;5;42m",
				43 => "\x1B[38;5;43m",
				44 => "\x1B[38;5;44m",
				45 => "\x1B[38;5;45m",
				46 => "\x1B[38;5;46m",
				47 => "\x1B[38;5;47m",
				48 => "\x1B[38;5;48m",
				49 => "\x1B[38;5;49m",
				50 => "\x1B[38;5;50m",
				51 => "\x1B[38;5;51m",
				52 => "\x1B[38;5;52m",
				53 => "\x1B[38;5;53m",
				54 => "\x1B[38;5;54m",
				55 => "\x1B[38;5;55m",
				56 => "\x1B[38;5;56m",
				57 => "\x1B[38;5;57m",
				58 => "\x1B[38;5;58m",
				59 => "\x1B[38;5;59m",
				60 => "\x1B[38;5;60m",
				61 => "\x1B[38;5;61m",
				62 => "\x1B[38;5;62m",
				63 => "\x1B[38;5;63m",
				64 => "\x1B[38;5;64m",
				65 => "\x1B[38;5;65m",
				66 => "\x1B[38;5;66m",
				67 => "\x1B[38;5;67m",
				68 => "\x1B[38;5;68m",
				69 => "\x1B[38;5;69m",
				70 => "\x1B[38;5;70m",
				71 => "\x1B[38;5;71m",
				72 => "\x1B[38;5;72m",
				73 => "\x1B[38;5;73m",
				74 => "\x1B[38;5;74m",
				75 => "\x1B[38;5;75m",
				76 => "\x1B[38;5;76m",
				77 => "\x1B[38;5;77m",
				78 => "\x1B[38;5;78m",
				79 => "\x1B[38;5;79m",
				80 => "\x1B[38;5;80m",
				81 => "\x1B[38;5;81m",
				82 => "\x1B[38;5;82m",
				83 => "\x1B[38;5;83m",
				84 => "\x1B[38;5;84m",
				85 => "\x1B[38;5;85m",
				86 => "\x1B[38;5;86m",
				87 => "\x1B[38;5;87m",
				88 => "\x1B[38;5;88m",
				89 => "\x1B[38;5;89m",
				90 => "\x1B[38;5;90m",
				91 => "\x1B[38;5;91m",
				92 => "\x1B[38;5;92m",
				93 => "\x1B[38;5;93m",
				94 => "\x1B[38;5;94m",
				95 => "\x1B[38;5;95m",
				96 => "\x1B[38;5;96m",
				97 => "\x1B[38;5;97m",
				98 => "\x1B[38;5;98m",
				99 => "\x1B[38;5;99m",
				100 => "\x1B[38;5;100m",
				101 => "\x1B[38;5;101m",
				102 => "\x1B[38;5;102m",
				103 => "\x1B[38;5;103m",
				104 => "\x1B[38;5;104m",
				105 => "\x1B[38;5;105m",
				106 => "\x1B[38;5;106m",
				107 => "\x1B[38;5;107m",
				108 => "\x1B[38;5;108m",
				109 => "\x1B[38;5;109m",
				110 => "\x1B[38;5;110m",
				111 => "\x1B[38;5;111m",
				112 => "\x1B[38;5;112m",
				113 => "\x1B[38;5;113m",
				114 => "\x1B[38;5;114m",
				115 => "\x1B[38;5;115m",
				116 => "\x1B[38;5;116m",
				117 => "\x1B[38;5;117m",
				118 => "\x1B[38;5;118m",
				119 => "\x1B[38;5;119m",
				120 => "\x1B[38;5;120m",
				121 => "\x1B[38;5;121m",
				122 => "\x1B[38;5;122m",
				123 => "\x1B[38;5;123m",
				124 => "\x1B[38;5;124m",
				125 => "\x1B[38;5;125m",
				126 => "\x1B[38;5;126m",
				127 => "\x1B[38;5;127m",
				128 => "\x1B[38;5;128m",
				129 => "\x1B[38;5;129m",
				130 => "\x1B[38;5;130m",
				131 => "\x1B[38;5;131m",
				132 => "\x1B[38;5;132m",
				133 => "\x1B[38;5;133m",
				134 => "\x1B[38;5;134m",
				135 => "\x1B[38;5;135m",
				136 => "\x1B[38;5;136m",
				137 => "\x1B[38;5;137m",
				138 => "\x1B[38;5;138m",
				139 => "\x1B[38;5;139m",
				140 => "\x1B[38;5;140m",
				141 => "\x1B[38;5;141m",
				142 => "\x1B[38;5;142m",
				143 => "\x1B[38;5;143m",
				144 => "\x1B[38;5;144m",
				145 => "\x1B[38;5;145m",
				146 => "\x1B[38;5;146m",
				147 => "\x1B[38;5;147m",
				148 => "\x1B[38;5;148m",
				149 => "\x1B[38;5;149m",
				150 => "\x1B[38;5;150m",
				151 => "\x1B[38;5;151m",
				152 => "\x1B[38;5;152m",
				153 => "\x1B[38;5;153m",
				154 => "\x1B[38;5;154m",
				155 => "\x1B[38;5;155m",
				156 => "\x1B[38;5;156m",
				157 => "\x1B[38;5;157m",
				158 => "\x1B[38;5;158m",
				159 => "\x1B[38;5;159m",
				160 => "\x1B[38;5;160m",
				161 => "\x1B[38;5;161m",
				162 => "\x1B[38;5;162m",
				163 => "\x1B[38;5;163m",
				164 => "\x1B[38;5;164m",
				165 => "\x1B[38;5;165m",
				166 => "\x1B[38;5;166m",
				167 => "\x1B[38;5;167m",
				168 => "\x1B[38;5;168m",
				169 => "\x1B[38;5;169m",
				170 => "\x1B[38;5;170m",
				171 => "\x1B[38;5;171m",
				172 => "\x1B[38;5;172m",
				173 => "\x1B[38;5;173m",
				174 => "\x1B[38;5;174m",
				175 => "\x1B[38;5;175m",
				176 => "\x1B[38;5;176m",
				177 => "\x1B[38;5;177m",
				178 => "\x1B[38;5;178m",
				179 => "\x1B[38;5;179m",
				180 => "\x1B[38;5;180m",
				181 => "\x1B[38;5;181m",
				182 => "\x1B[38;5;182m",
				183 => "\x1B[38;5;183m",
				184 => "\x1B[38;5;184m",
				185 => "\x1B[38;5;185m",
				186 => "\x1B[38;5;186m",
				187 => "\x1B[38;5;187m",
				188 => "\x1B[38;5;188m",
				189 => "\x1B[38;5;189m",
				190 => "\x1B[38;5;190m",
				191 => "\x1B[38;5;191m",
				192 => "\x1B[38;5;192m",
				193 => "\x1B[38;5;193m",
				194 => "\x1B[38;5;194m",
				195 => "\x1B[38;5;195m",
				196 => "\x1B[38;5;196m",
				197 => "\x1B[38;5;197m",
				198 => "\x1B[38;5;198m",
				199 => "\x1B[38;5;199m",
				200 => "\x1B[38;5;200m",
				201 => "\x1B[38;5;201m",
				202 => "\x1B[38;5;202m",
				203 => "\x1B[38;5;203m",
				204 => "\x1B[38;5;204m",
				205 => "\x1B[38;5;205m",
				206 => "\x1B[38;5;206m",
				207 => "\x1B[38;5;207m",
				208 => "\x1B[38;5;208m",
				209 => "\x1B[38;5;209m",
				210 => "\x1B[38;5;210m",
				211 => "\x1B[38;5;211m",
				212 => "\x1B[38;5;212m",
				213 => "\x1B[38;5;213m",
				214 => "\x1B[38;5;214m",
				215 => "\x1B[38;5;215m",
				216 => "\x1B[38;5;216m",
				217 => "\x1B[38;5;217m",
				218 => "\x1B[38;5;218m",
				219 => "\x1B[38;5;219m",
				220 => "\x1B[38;5;220m",
				221 => "\x1B[38;5;221m",
				222 => "\x1B[38;5;222m",
				223 => "\x1B[38;5;223m",
				224 => "\x1B[38;5;224m",
				225 => "\x1B[38;5;225m",
				226 => "\x1B[38;5;226m",
				227 => "\x1B[38;5;227m",
				228 => "\x1B[38;5;228m",
				229 => "\x1B[38;5;229m",
				230 => "\x1B[38;5;230m",
				231 => "\x1B[38;5;231m",
				232 => "\x1B[38;5;232m",
				233 => "\x1B[38;5;233m",
				234 => "\x1B[38;5;234m",
				235 => "\x1B[38;5;235m",
				236 => "\x1B[38;5;236m",
				237 => "\x1B[38;5;237m",
				238 => "\x1B[38;5;238m",
				239 => "\x1B[38;5;239m",
				240 => "\x1B[38;5;240m",
				241 => "\x1B[38;5;241m",
				242 => "\x1B[38;5;242m",
				243 => "\x1B[38;5;243m",
				244 => "\x1B[38;5;244m",
				245 => "\x1B[38;5;245m",
				246 => "\x1B[38;5;246m",
				247 => "\x1B[38;5;247m",
				248 => "\x1B[38;5;248m",
				249 => "\x1B[38;5;249m",
				250 => "\x1B[38;5;250m",
				251 => "\x1B[38;5;251m",
				252 => "\x1B[38;5;252m",
				253 => "\x1B[38;5;253m",
				254 => "\x1B[38;5;254m",
				255 => "\x1B[38;5;255m",
			},
			Self::Bold(c) => match c {
				0 => "",
				1 => "\x1B[1;38;5;1m",
				2 => "\x1B[1;38;5;2m",
				3 => "\x1B[1;38;5;3m",
				4 => "\x1B[1;38;5;4m",
				5 => "\x1B[1;38;5;5m",
				6 => "\x1B[1;38;5;6m",
				7 => "\x1B[1;38;5;7m",
				8 => "\x1B[1;38;5;8m",
				9 => "\x1B[1;38;5;9m",
				10 => "\x1B[1;38;5;10m",
				11 => "\x1B[1;38;5;11m",
				12 => "\x1B[1;38;5;12m",
				13 => "\x1B[1;38;5;13m",
				14 => "\x1B[1;38;5;14m",
				15 => "\x1B[1;38;5;15m",
				16 => "\x1B[1;38;5;16m",
				17 => "\x1B[1;38;5;17m",
				18 => "\x1B[1;38;5;18m",
				19 => "\x1B[1;38;5;19m",
				20 => "\x1B[1;38;5;20m",
				21 => "\x1B[1;38;5;21m",
				22 => "\x1B[1;38;5;22m",
				23 => "\x1B[1;38;5;23m",
				24 => "\x1B[1;38;5;24m",
				25 => "\x1B[1;38;5;25m",
				26 => "\x1B[1;38;5;26m",
				27 => "\x1B[1;38;5;27m",
				28 => "\x1B[1;38;5;28m",
				29 => "\x1B[1;38;5;29m",
				30 => "\x1B[1;38;5;30m",
				31 => "\x1B[1;38;5;31m",
				32 => "\x1B[1;38;5;32m",
				33 => "\x1B[1;38;5;33m",
				34 => "\x1B[1;38;5;34m",
				35 => "\x1B[1;38;5;35m",
				36 => "\x1B[1;38;5;36m",
				37 => "\x1B[1;38;5;37m",
				38 => "\x1B[1;38;5;38m",
				39 => "\x1B[1;38;5;39m",
				40 => "\x1B[1;38;5;40m",
				41 => "\x1B[1;38;5;41m",
				42 => "\x1B[1;38;5;42m",
				43 => "\x1B[1;38;5;43m",
				44 => "\x1B[1;38;5;44m",
				45 => "\x1B[1;38;5;45m",
				46 => "\x1B[1;38;5;46m",
				47 => "\x1B[1;38;5;47m",
				48 => "\x1B[1;38;5;48m",
				49 => "\x1B[1;38;5;49m",
				50 => "\x1B[1;38;5;50m",
				51 => "\x1B[1;38;5;51m",
				52 => "\x1B[1;38;5;52m",
				53 => "\x1B[1;38;5;53m",
				54 => "\x1B[1;38;5;54m",
				55 => "\x1B[1;38;5;55m",
				56 => "\x1B[1;38;5;56m",
				57 => "\x1B[1;38;5;57m",
				58 => "\x1B[1;38;5;58m",
				59 => "\x1B[1;38;5;59m",
				60 => "\x1B[1;38;5;60m",
				61 => "\x1B[1;38;5;61m",
				62 => "\x1B[1;38;5;62m",
				63 => "\x1B[1;38;5;63m",
				64 => "\x1B[1;38;5;64m",
				65 => "\x1B[1;38;5;65m",
				66 => "\x1B[1;38;5;66m",
				67 => "\x1B[1;38;5;67m",
				68 => "\x1B[1;38;5;68m",
				69 => "\x1B[1;38;5;69m",
				70 => "\x1B[1;38;5;70m",
				71 => "\x1B[1;38;5;71m",
				72 => "\x1B[1;38;5;72m",
				73 => "\x1B[1;38;5;73m",
				74 => "\x1B[1;38;5;74m",
				75 => "\x1B[1;38;5;75m",
				76 => "\x1B[1;38;5;76m",
				77 => "\x1B[1;38;5;77m",
				78 => "\x1B[1;38;5;78m",
				79 => "\x1B[1;38;5;79m",
				80 => "\x1B[1;38;5;80m",
				81 => "\x1B[1;38;5;81m",
				82 => "\x1B[1;38;5;82m",
				83 => "\x1B[1;38;5;83m",
				84 => "\x1B[1;38;5;84m",
				85 => "\x1B[1;38;5;85m",
				86 => "\x1B[1;38;5;86m",
				87 => "\x1B[1;38;5;87m",
				88 => "\x1B[1;38;5;88m",
				89 => "\x1B[1;38;5;89m",
				90 => "\x1B[1;38;5;90m",
				91 => "\x1B[1;38;5;91m",
				92 => "\x1B[1;38;5;92m",
				93 => "\x1B[1;38;5;93m",
				94 => "\x1B[1;38;5;94m",
				95 => "\x1B[1;38;5;95m",
				96 => "\x1B[1;38;5;96m",
				97 => "\x1B[1;38;5;97m",
				98 => "\x1B[1;38;5;98m",
				99 => "\x1B[1;38;5;99m",
				100 => "\x1B[1;38;5;100m",
				101 => "\x1B[1;38;5;101m",
				102 => "\x1B[1;38;5;102m",
				103 => "\x1B[1;38;5;103m",
				104 => "\x1B[1;38;5;104m",
				105 => "\x1B[1;38;5;105m",
				106 => "\x1B[1;38;5;106m",
				107 => "\x1B[1;38;5;107m",
				108 => "\x1B[1;38;5;108m",
				109 => "\x1B[1;38;5;109m",
				110 => "\x1B[1;38;5;110m",
				111 => "\x1B[1;38;5;111m",
				112 => "\x1B[1;38;5;112m",
				113 => "\x1B[1;38;5;113m",
				114 => "\x1B[1;38;5;114m",
				115 => "\x1B[1;38;5;115m",
				116 => "\x1B[1;38;5;116m",
				117 => "\x1B[1;38;5;117m",
				118 => "\x1B[1;38;5;118m",
				119 => "\x1B[1;38;5;119m",
				120 => "\x1B[1;38;5;120m",
				121 => "\x1B[1;38;5;121m",
				122 => "\x1B[1;38;5;122m",
				123 => "\x1B[1;38;5;123m",
				124 => "\x1B[1;38;5;124m",
				125 => "\x1B[1;38;5;125m",
				126 => "\x1B[1;38;5;126m",
				127 => "\x1B[1;38;5;127m",
				128 => "\x1B[1;38;5;128m",
				129 => "\x1B[1;38;5;129m",
				130 => "\x1B[1;38;5;130m",
				131 => "\x1B[1;38;5;131m",
				132 => "\x1B[1;38;5;132m",
				133 => "\x1B[1;38;5;133m",
				134 => "\x1B[1;38;5;134m",
				135 => "\x1B[1;38;5;135m",
				136 => "\x1B[1;38;5;136m",
				137 => "\x1B[1;38;5;137m",
				138 => "\x1B[1;38;5;138m",
				139 => "\x1B[1;38;5;139m",
				140 => "\x1B[1;38;5;140m",
				141 => "\x1B[1;38;5;141m",
				142 => "\x1B[1;38;5;142m",
				143 => "\x1B[1;38;5;143m",
				144 => "\x1B[1;38;5;144m",
				145 => "\x1B[1;38;5;145m",
				146 => "\x1B[1;38;5;146m",
				147 => "\x1B[1;38;5;147m",
				148 => "\x1B[1;38;5;148m",
				149 => "\x1B[1;38;5;149m",
				150 => "\x1B[1;38;5;150m",
				151 => "\x1B[1;38;5;151m",
				152 => "\x1B[1;38;5;152m",
				153 => "\x1B[1;38;5;153m",
				154 => "\x1B[1;38;5;154m",
				155 => "\x1B[1;38;5;155m",
				156 => "\x1B[1;38;5;156m",
				157 => "\x1B[1;38;5;157m",
				158 => "\x1B[1;38;5;158m",
				159 => "\x1B[1;38;5;159m",
				160 => "\x1B[1;38;5;160m",
				161 => "\x1B[1;38;5;161m",
				162 => "\x1B[1;38;5;162m",
				163 => "\x1B[1;38;5;163m",
				164 => "\x1B[1;38;5;164m",
				165 => "\x1B[1;38;5;165m",
				166 => "\x1B[1;38;5;166m",
				167 => "\x1B[1;38;5;167m",
				168 => "\x1B[1;38;5;168m",
				169 => "\x1B[1;38;5;169m",
				170 => "\x1B[1;38;5;170m",
				171 => "\x1B[1;38;5;171m",
				172 => "\x1B[1;38;5;172m",
				173 => "\x1B[1;38;5;173m",
				174 => "\x1B[1;38;5;174m",
				175 => "\x1B[1;38;5;175m",
				176 => "\x1B[1;38;5;176m",
				177 => "\x1B[1;38;5;177m",
				178 => "\x1B[1;38;5;178m",
				179 => "\x1B[1;38;5;179m",
				180 => "\x1B[1;38;5;180m",
				181 => "\x1B[1;38;5;181m",
				182 => "\x1B[1;38;5;182m",
				183 => "\x1B[1;38;5;183m",
				184 => "\x1B[1;38;5;184m",
				185 => "\x1B[1;38;5;185m",
				186 => "\x1B[1;38;5;186m",
				187 => "\x1B[1;38;5;187m",
				188 => "\x1B[1;38;5;188m",
				189 => "\x1B[1;38;5;189m",
				190 => "\x1B[1;38;5;190m",
				191 => "\x1B[1;38;5;191m",
				192 => "\x1B[1;38;5;192m",
				193 => "\x1B[1;38;5;193m",
				194 => "\x1B[1;38;5;194m",
				195 => "\x1B[1;38;5;195m",
				196 => "\x1B[1;38;5;196m",
				197 => "\x1B[1;38;5;197m",
				198 => "\x1B[1;38;5;198m",
				199 => "\x1B[1;38;5;199m",
				200 => "\x1B[1;38;5;200m",
				201 => "\x1B[1;38;5;201m",
				202 => "\x1B[1;38;5;202m",
				203 => "\x1B[1;38;5;203m",
				204 => "\x1B[1;38;5;204m",
				205 => "\x1B[1;38;5;205m",
				206 => "\x1B[1;38;5;206m",
				207 => "\x1B[1;38;5;207m",
				208 => "\x1B[1;38;5;208m",
				209 => "\x1B[1;38;5;209m",
				210 => "\x1B[1;38;5;210m",
				211 => "\x1B[1;38;5;211m",
				212 => "\x1B[1;38;5;212m",
				213 => "\x1B[1;38;5;213m",
				214 => "\x1B[1;38;5;214m",
				215 => "\x1B[1;38;5;215m",
				216 => "\x1B[1;38;5;216m",
				217 => "\x1B[1;38;5;217m",
				218 => "\x1B[1;38;5;218m",
				219 => "\x1B[1;38;5;219m",
				220 => "\x1B[1;38;5;220m",
				221 => "\x1B[1;38;5;221m",
				222 => "\x1B[1;38;5;222m",
				223 => "\x1B[1;38;5;223m",
				224 => "\x1B[1;38;5;224m",
				225 => "\x1B[1;38;5;225m",
				226 => "\x1B[1;38;5;226m",
				227 => "\x1B[1;38;5;227m",
				228 => "\x1B[1;38;5;228m",
				229 => "\x1B[1;38;5;229m",
				230 => "\x1B[1;38;5;230m",
				231 => "\x1B[1;38;5;231m",
				232 => "\x1B[1;38;5;232m",
				233 => "\x1B[1;38;5;233m",
				234 => "\x1B[1;38;5;234m",
				235 => "\x1B[1;38;5;235m",
				236 => "\x1B[1;38;5;236m",
				237 => "\x1B[1;38;5;237m",
				238 => "\x1B[1;38;5;238m",
				239 => "\x1B[1;38;5;239m",
				240 => "\x1B[1;38;5;240m",
				241 => "\x1B[1;38;5;241m",
				242 => "\x1B[1;38;5;242m",
				243 => "\x1B[1;38;5;243m",
				244 => "\x1B[1;38;5;244m",
				245 => "\x1B[1;38;5;245m",
				246 => "\x1B[1;38;5;246m",
				247 => "\x1B[1;38;5;247m",
				248 => "\x1B[1;38;5;248m",
				249 => "\x1B[1;38;5;249m",
				250 => "\x1B[1;38;5;250m",
				251 => "\x1B[1;38;5;251m",
				252 => "\x1B[1;38;5;252m",
				253 => "\x1B[1;38;5;253m",
				254 => "\x1B[1;38;5;254m",
				255 => "\x1B[1;38;5;255m",
			},
			Self::Dim(c) => match c {
				0 => "",
				1 => "\x1B[2;38;5;1m",
				2 => "\x1B[2;38;5;2m",
				3 => "\x1B[2;38;5;3m",
				4 => "\x1B[2;38;5;4m",
				5 => "\x1B[2;38;5;5m",
				6 => "\x1B[2;38;5;6m",
				7 => "\x1B[2;38;5;7m",
				8 => "\x1B[2;38;5;8m",
				9 => "\x1B[2;38;5;9m",
				10 => "\x1B[2;38;5;10m",
				11 => "\x1B[2;38;5;11m",
				12 => "\x1B[2;38;5;12m",
				13 => "\x1B[2;38;5;13m",
				14 => "\x1B[2;38;5;14m",
				15 => "\x1B[2;38;5;15m",
				16 => "\x1B[2;38;5;16m",
				17 => "\x1B[2;38;5;17m",
				18 => "\x1B[2;38;5;18m",
				19 => "\x1B[2;38;5;19m",
				20 => "\x1B[2;38;5;20m",
				21 => "\x1B[2;38;5;21m",
				22 => "\x1B[2;38;5;22m",
				23 => "\x1B[2;38;5;23m",
				24 => "\x1B[2;38;5;24m",
				25 => "\x1B[2;38;5;25m",
				26 => "\x1B[2;38;5;26m",
				27 => "\x1B[2;38;5;27m",
				28 => "\x1B[2;38;5;28m",
				29 => "\x1B[2;38;5;29m",
				30 => "\x1B[2;38;5;30m",
				31 => "\x1B[2;38;5;31m",
				32 => "\x1B[2;38;5;32m",
				33 => "\x1B[2;38;5;33m",
				34 => "\x1B[2;38;5;34m",
				35 => "\x1B[2;38;5;35m",
				36 => "\x1B[2;38;5;36m",
				37 => "\x1B[2;38;5;37m",
				38 => "\x1B[2;38;5;38m",
				39 => "\x1B[2;38;5;39m",
				40 => "\x1B[2;38;5;40m",
				41 => "\x1B[2;38;5;41m",
				42 => "\x1B[2;38;5;42m",
				43 => "\x1B[2;38;5;43m",
				44 => "\x1B[2;38;5;44m",
				45 => "\x1B[2;38;5;45m",
				46 => "\x1B[2;38;5;46m",
				47 => "\x1B[2;38;5;47m",
				48 => "\x1B[2;38;5;48m",
				49 => "\x1B[2;38;5;49m",
				50 => "\x1B[2;38;5;50m",
				51 => "\x1B[2;38;5;51m",
				52 => "\x1B[2;38;5;52m",
				53 => "\x1B[2;38;5;53m",
				54 => "\x1B[2;38;5;54m",
				55 => "\x1B[2;38;5;55m",
				56 => "\x1B[2;38;5;56m",
				57 => "\x1B[2;38;5;57m",
				58 => "\x1B[2;38;5;58m",
				59 => "\x1B[2;38;5;59m",
				60 => "\x1B[2;38;5;60m",
				61 => "\x1B[2;38;5;61m",
				62 => "\x1B[2;38;5;62m",
				63 => "\x1B[2;38;5;63m",
				64 => "\x1B[2;38;5;64m",
				65 => "\x1B[2;38;5;65m",
				66 => "\x1B[2;38;5;66m",
				67 => "\x1B[2;38;5;67m",
				68 => "\x1B[2;38;5;68m",
				69 => "\x1B[2;38;5;69m",
				70 => "\x1B[2;38;5;70m",
				71 => "\x1B[2;38;5;71m",
				72 => "\x1B[2;38;5;72m",
				73 => "\x1B[2;38;5;73m",
				74 => "\x1B[2;38;5;74m",
				75 => "\x1B[2;38;5;75m",
				76 => "\x1B[2;38;5;76m",
				77 => "\x1B[2;38;5;77m",
				78 => "\x1B[2;38;5;78m",
				79 => "\x1B[2;38;5;79m",
				80 => "\x1B[2;38;5;80m",
				81 => "\x1B[2;38;5;81m",
				82 => "\x1B[2;38;5;82m",
				83 => "\x1B[2;38;5;83m",
				84 => "\x1B[2;38;5;84m",
				85 => "\x1B[2;38;5;85m",
				86 => "\x1B[2;38;5;86m",
				87 => "\x1B[2;38;5;87m",
				88 => "\x1B[2;38;5;88m",
				89 => "\x1B[2;38;5;89m",
				90 => "\x1B[2;38;5;90m",
				91 => "\x1B[2;38;5;91m",
				92 => "\x1B[2;38;5;92m",
				93 => "\x1B[2;38;5;93m",
				94 => "\x1B[2;38;5;94m",
				95 => "\x1B[2;38;5;95m",
				96 => "\x1B[2;38;5;96m",
				97 => "\x1B[2;38;5;97m",
				98 => "\x1B[2;38;5;98m",
				99 => "\x1B[2;38;5;99m",
				100 => "\x1B[2;38;5;100m",
				101 => "\x1B[2;38;5;101m",
				102 => "\x1B[2;38;5;102m",
				103 => "\x1B[2;38;5;103m",
				104 => "\x1B[2;38;5;104m",
				105 => "\x1B[2;38;5;105m",
				106 => "\x1B[2;38;5;106m",
				107 => "\x1B[2;38;5;107m",
				108 => "\x1B[2;38;5;108m",
				109 => "\x1B[2;38;5;109m",
				110 => "\x1B[2;38;5;110m",
				111 => "\x1B[2;38;5;111m",
				112 => "\x1B[2;38;5;112m",
				113 => "\x1B[2;38;5;113m",
				114 => "\x1B[2;38;5;114m",
				115 => "\x1B[2;38;5;115m",
				116 => "\x1B[2;38;5;116m",
				117 => "\x1B[2;38;5;117m",
				118 => "\x1B[2;38;5;118m",
				119 => "\x1B[2;38;5;119m",
				120 => "\x1B[2;38;5;120m",
				121 => "\x1B[2;38;5;121m",
				122 => "\x1B[2;38;5;122m",
				123 => "\x1B[2;38;5;123m",
				124 => "\x1B[2;38;5;124m",
				125 => "\x1B[2;38;5;125m",
				126 => "\x1B[2;38;5;126m",
				127 => "\x1B[2;38;5;127m",
				128 => "\x1B[2;38;5;128m",
				129 => "\x1B[2;38;5;129m",
				130 => "\x1B[2;38;5;130m",
				131 => "\x1B[2;38;5;131m",
				132 => "\x1B[2;38;5;132m",
				133 => "\x1B[2;38;5;133m",
				134 => "\x1B[2;38;5;134m",
				135 => "\x1B[2;38;5;135m",
				136 => "\x1B[2;38;5;136m",
				137 => "\x1B[2;38;5;137m",
				138 => "\x1B[2;38;5;138m",
				139 => "\x1B[2;38;5;139m",
				140 => "\x1B[2;38;5;140m",
				141 => "\x1B[2;38;5;141m",
				142 => "\x1B[2;38;5;142m",
				143 => "\x1B[2;38;5;143m",
				144 => "\x1B[2;38;5;144m",
				145 => "\x1B[2;38;5;145m",
				146 => "\x1B[2;38;5;146m",
				147 => "\x1B[2;38;5;147m",
				148 => "\x1B[2;38;5;148m",
				149 => "\x1B[2;38;5;149m",
				150 => "\x1B[2;38;5;150m",
				151 => "\x1B[2;38;5;151m",
				152 => "\x1B[2;38;5;152m",
				153 => "\x1B[2;38;5;153m",
				154 => "\x1B[2;38;5;154m",
				155 => "\x1B[2;38;5;155m",
				156 => "\x1B[2;38;5;156m",
				157 => "\x1B[2;38;5;157m",
				158 => "\x1B[2;38;5;158m",
				159 => "\x1B[2;38;5;159m",
				160 => "\x1B[2;38;5;160m",
				161 => "\x1B[2;38;5;161m",
				162 => "\x1B[2;38;5;162m",
				163 => "\x1B[2;38;5;163m",
				164 => "\x1B[2;38;5;164m",
				165 => "\x1B[2;38;5;165m",
				166 => "\x1B[2;38;5;166m",
				167 => "\x1B[2;38;5;167m",
				168 => "\x1B[2;38;5;168m",
				169 => "\x1B[2;38;5;169m",
				170 => "\x1B[2;38;5;170m",
				171 => "\x1B[2;38;5;171m",
				172 => "\x1B[2;38;5;172m",
				173 => "\x1B[2;38;5;173m",
				174 => "\x1B[2;38;5;174m",
				175 => "\x1B[2;38;5;175m",
				176 => "\x1B[2;38;5;176m",
				177 => "\x1B[2;38;5;177m",
				178 => "\x1B[2;38;5;178m",
				179 => "\x1B[2;38;5;179m",
				180 => "\x1B[2;38;5;180m",
				181 => "\x1B[2;38;5;181m",
				182 => "\x1B[2;38;5;182m",
				183 => "\x1B[2;38;5;183m",
				184 => "\x1B[2;38;5;184m",
				185 => "\x1B[2;38;5;185m",
				186 => "\x1B[2;38;5;186m",
				187 => "\x1B[2;38;5;187m",
				188 => "\x1B[2;38;5;188m",
				189 => "\x1B[2;38;5;189m",
				190 => "\x1B[2;38;5;190m",
				191 => "\x1B[2;38;5;191m",
				192 => "\x1B[2;38;5;192m",
				193 => "\x1B[2;38;5;193m",
				194 => "\x1B[2;38;5;194m",
				195 => "\x1B[2;38;5;195m",
				196 => "\x1B[2;38;5;196m",
				197 => "\x1B[2;38;5;197m",
				198 => "\x1B[2;38;5;198m",
				199 => "\x1B[2;38;5;199m",
				200 => "\x1B[2;38;5;200m",
				201 => "\x1B[2;38;5;201m",
				202 => "\x1B[2;38;5;202m",
				203 => "\x1B[2;38;5;203m",
				204 => "\x1B[2;38;5;204m",
				205 => "\x1B[2;38;5;205m",
				206 => "\x1B[2;38;5;206m",
				207 => "\x1B[2;38;5;207m",
				208 => "\x1B[2;38;5;208m",
				209 => "\x1B[2;38;5;209m",
				210 => "\x1B[2;38;5;210m",
				211 => "\x1B[2;38;5;211m",
				212 => "\x1B[2;38;5;212m",
				213 => "\x1B[2;38;5;213m",
				214 => "\x1B[2;38;5;214m",
				215 => "\x1B[2;38;5;215m",
				216 => "\x1B[2;38;5;216m",
				217 => "\x1B[2;38;5;217m",
				218 => "\x1B[2;38;5;218m",
				219 => "\x1B[2;38;5;219m",
				220 => "\x1B[2;38;5;220m",
				221 => "\x1B[2;38;5;221m",
				222 => "\x1B[2;38;5;222m",
				223 => "\x1B[2;38;5;223m",
				224 => "\x1B[2;38;5;224m",
				225 => "\x1B[2;38;5;225m",
				226 => "\x1B[2;38;5;226m",
				227 => "\x1B[2;38;5;227m",
				228 => "\x1B[2;38;5;228m",
				229 => "\x1B[2;38;5;229m",
				230 => "\x1B[2;38;5;230m",
				231 => "\x1B[2;38;5;231m",
				232 => "\x1B[2;38;5;232m",
				233 => "\x1B[2;38;5;233m",
				234 => "\x1B[2;38;5;234m",
				235 => "\x1B[2;38;5;235m",
				236 => "\x1B[2;38;5;236m",
				237 => "\x1B[2;38;5;237m",
				238 => "\x1B[2;38;5;238m",
				239 => "\x1B[2;38;5;239m",
				240 => "\x1B[2;38;5;240m",
				241 => "\x1B[2;38;5;241m",
				242 => "\x1B[2;38;5;242m",
				243 => "\x1B[2;38;5;243m",
				244 => "\x1B[2;38;5;244m",
				245 => "\x1B[2;38;5;245m",
				246 => "\x1B[2;38;5;246m",
				247 => "\x1B[2;38;5;247m",
				248 => "\x1B[2;38;5;248m",
				249 => "\x1B[2;38;5;249m",
				250 => "\x1B[2;38;5;250m",
				251 => "\x1B[2;38;5;251m",
				252 => "\x1B[2;38;5;252m",
				253 => "\x1B[2;38;5;253m",
				254 => "\x1B[2;38;5;254m",
				255 => "\x1B[2;38;5;255m",
			},
		}
	}
}



#[derive(Debug, Clone, PartialEq)]
/// Generic message.
pub enum Prefix<'mp> {
	/// Custom.
	Custom(Cow<'mp, str>),
	/// Debug.
	Debug,
	/// Error.
	Error,
	/// Info.
	Info,
	/// Notice.
	Notice,
	/// Success.
	Success,
	/// Warning.
	Warning,
	/// None.
	None,
}

impl AsRef<str> for Prefix<'_> {
	/// As Ref String.
	fn as_ref(&self) -> &str {
		&**self
	}
}

impl Default for Prefix<'_> {
	/// Default.
	fn default() -> Self {
		Prefix::None
	}
}

impl Deref for Prefix<'_> {
	type Target = str;

	/// Deref.
	fn deref(&self) -> &Self::Target {
		match *self {
			Self::Custom(ref p) => p,
			Self::Debug => "\x1B[96;1mDebug:\x1B[0m ",
			Self::Error => "\x1B[91;1mError:\x1B[0m ",
			Self::Info => "\x1B[96;1mInfo:\x1B[0m ",
			Self::Notice => "\x1B[95;1mNotice:\x1B[0m ",
			Self::Success => "\x1B[92;1mSuccess:\x1B[0m ",
			Self::Warning => "\x1B[93;1mWarning:\x1B[0m ",
			_ => "",
		}
	}
}

impl fmt::Display for Prefix<'_> {
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}

impl<'mp> Prefix<'mp> {
	/// Custom Prefix.
	pub fn new<S> (msg: S, color: u8) -> Self
	where S: Into<Cow<'mp, str>> {
		let msg = msg.into();
		if msg.is_empty() {
			Self::None
		}
		else {
			let mut out: String = String::with_capacity(msg.len() + 19);
			out.push_str(&Color::Bold(color));
			out.push_str(&msg);
			out.push_str(":\x1B[0m ");
			Self::Custom(out.into())
		}
	}

	#[must_use]
	/// Happy or sad?
	pub fn happy(&self) -> bool {
		match *self {
			Self::Error | Self::Warning => false,
			_ => true,
		}
	}

	#[must_use]
	/// Print the prefix!
	pub fn as_bytes(&'mp self) -> &'mp [u8] {
		match *self {
			Self::Custom(ref p) => p.as_bytes(),
			Self::Debug => b"\x1B[96;1mDebug:\x1B[0m ",
			Self::Error => b"\x1B[91;1mError:\x1B[0m ",
			Self::Info => b"\x1B[96;1mInfo:\x1B[0m ",
			Self::Notice => b"\x1B[95;1mNotice:\x1B[0m ",
			Self::Success => b"\x1B[92;1mSuccess:\x1B[0m ",
			Self::Warning => b"\x1B[93;1mWarning:\x1B[0m ",
			_ => b"",
		}
	}

	#[must_use]
	/// Print the prefix!
	pub fn prefix(&'mp self) -> Cow<'mp, str> {
		(&**self).into()
	}

	#[must_use]
	/// Prefix length.
	pub fn is_empty(&'mp self) -> bool {
		Self::None == *self
	}

	#[must_use]
	/// Prefix length.
	pub fn len(&'mp self) -> usize {
		match *self {
			Self::Custom(ref p) => p.len(),
			Self::Debug | Self::Error => 18,
			Self::Info => 17,
			Self::Notice => 19,
			Self::Success | Self::Warning => 20,
			_ => 0,
		}
	}

	#[must_use]
	/// Prefix width.
	pub fn width(&'mp self) -> usize {
		match *self {
			Self::Custom(ref p) => p.width(),
			Self::Debug | Self::Error => 7,
			Self::Info => 6,
			Self::Notice => 8,
			Self::Success | Self::Warning => 9,
			_ => 0,
		}
	}
}



#[derive(Debug, Default, Clone)]
/// Message.
pub struct Msg<'m> {
	indent: u8,
	prefix: Prefix<'m>,
	msg: Cow<'m, str>,
	flags: u8,
}

impl fmt::Display for Msg<'_> {
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.msg())
	}
}

impl<'m> Msg<'m> {
	/// New.
	pub fn new<S> (msg: S) -> Self
	where S: Into<Cow<'m, str>> {
		Msg {
			msg: msg.into(),
			..Msg::default()
		}
	}

	#[must_use]
	/// Set Flags.
	pub fn with_flags(mut self, flags: u8) -> Self {
		self.flags = flags;
		self
	}

	#[must_use]
	/// Set Indentation.
	pub fn with_indent(mut self, indent: u8) -> Self {
		self.indent = indent;
		self
	}

	#[must_use]
	/// Set Prefix.
	pub fn with_prefix(mut self, prefix: Prefix<'m>) -> Self {
		self.prefix = prefix;
		self
	}



	// -------------------------------------------------------------
	// Getters
	// -------------------------------------------------------------

	#[must_use]
	/// Msg.
	pub fn msg(&self) -> Cow<'_, str> {
		let mut buf: BytesMut = BytesMut::with_capacity(256);

		if 0 != self.indent {
			self._msg_put_indent(&mut buf);
		}
		if ! self.prefix.is_empty() {
			self._msg_put_prefix(&mut buf);
		}
		if ! self.msg.is_empty() {
			self._msg_put_msg(&mut buf);
		}
		if 0 != (MSG_TIMESTAMP & self.flags) {
			self._msg_put_timestamp(&mut buf);
		}

		Cow::Owned(unsafe { String::from_utf8_unchecked(buf.to_vec()) })
	}

	/// Msg indent.
	fn _msg_put_indent(&self, buf: &mut BytesMut) {
		let indent: usize = self.indent as usize * 4;
		buf.put(strings::whitespace_bytes(indent).as_ref());
	}

	/// Msg prefix.
	fn _msg_put_prefix(&self, buf: &mut BytesMut) {
		buf.put(self.prefix.as_bytes());
	}

	/// Msg.
	fn _msg_put_msg(&self, buf: &mut BytesMut) {
		buf.extend_from_slice(b"\x1B[1m");
		buf.put(self.msg.as_bytes());
		buf.extend_from_slice(b"\x1B[0m");
	}

	/// Timestamp.
	fn _msg_put_timestamp(&self, buf: &mut BytesMut) {
		let width: usize = cli::term_width();
		let old_width: usize = buf.width();

		// Can it fit on one line?
		if width > old_width + 21 {
			buf.put(strings::whitespace_bytes(width - 21 - old_width).as_ref());
			buf.extend_from_slice(b"\x1B[2m[\x1B[34;2m");
			Msg::_msg_put_timestamp_inner(buf);
			buf.extend_from_slice(b"\x1B[0m\x1B[2m]\x1B[0m");
		}
		// Well shit.
		else {
			let b = buf.split();
			if 0 != self.indent {
				self._msg_put_indent(buf);
			}
			buf.extend_from_slice(b"\x1B[2m[\x1B[34;2m");
			Msg::_msg_put_timestamp_inner(buf);
			buf.extend_from_slice(b"\x1B[0m\x1B[2m]\x1B[0m\n");
			buf.unsplit(b);
		}
	}

	/// Write *just* the timestamp part.
	fn _msg_put_timestamp_inner(buf: &mut BytesMut) {
		use chrono::{
			Datelike,
			Local,
			Timelike,
		};

		let now = Local::now();
		itoa::fmt(&mut *buf, now.year()).expect("Invalid year.");
		buf.put_u8(b'-');
		buf.extend_from_slice(strings::zero_padded_time_bytes(now.month()));
		buf.put_u8(b'-');
		buf.extend_from_slice(strings::zero_padded_time_bytes(now.day()));
		buf.put_u8(b' ');
		buf.extend_from_slice(strings::zero_padded_time_bytes(now.hour()));
		buf.put_u8(b'-');
		buf.extend_from_slice(strings::zero_padded_time_bytes(now.minute()));
		buf.put_u8(b'-');
		buf.extend_from_slice(strings::zero_padded_time_bytes(now.second()));
	}

	#[must_use]
	/// Prefix.
	pub fn prefix(&self) -> Prefix {
		self.prefix.clone()
	}



	// -------------------------------------------------------------
	// Misc Operations
	// -------------------------------------------------------------

	#[cfg(feature = "interactive")]
	#[must_use]
	/// Prompt instead.
	pub fn prompt(&self) -> bool {
		casual::confirm(&[
			"\x1B[93;1mConfirm:\x1B[0m \x1B[1m",
			&self.msg,
			"\x1B[0m",
		].concat())
	}

	/// Print.
	pub fn print(&self) {
		let mut flags: u8 = self.flags | PRINT_NEWLINE;
		if ! self.prefix.happy() {
			flags |= PRINT_STDERR;
		}

		cli::print(self.msg(), flags);
	}

	// -------------------------------------------------------------
	// Message Templates
	// -------------------------------------------------------------

	#[must_use]
	/// Template: Crunched In X.
	pub fn crunched_in(num: u64, time: Instant, du: Option<(u64, u64)>) -> Self {
		let elapsed = time.elapsed().as_secs().elapsed();

		Msg::new(Cow::Owned(match du {
			Some((before, after)) => [
				num.inflect("file", "files").as_ref(),
				" in ",
				&elapsed,
				&match before.saved(after) {
					0 => ", but no dice".to_string(),
					x => format!(
						", saving {} bytes ({:3.*}%)",
						x.to_formatted_string(&Locale::en),
						2,
						(1.0 - (after as f64 / before as f64)) * 100.0
					),
				},
				".",
			].concat(),
			None => [
				num.inflect("file", "files").as_ref(),
				" in ",
				&elapsed,
				".",
			].concat(),
		}))
			.with_prefix(Prefix::new("Crunched", 2))
	}

	#[must_use]
	/// Template: Finished In X.
	pub fn finished_in(time: Instant) -> Self {
		Msg::new(Cow::Owned([
			"Finished in ",
			&time.elapsed().as_secs().elapsed(),
			".",
		].concat()))
			.with_prefix(Prefix::new("Crunched", 2))
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn prefix() {
		assert_eq!(Prefix::Debug.prefix().strip_ansi(), "Debug: ");
		assert_eq!(Prefix::Error.prefix().strip_ansi(), "Error: ");
		assert_eq!(Prefix::Info.prefix().strip_ansi(), "Info: ");
		assert_eq!(Prefix::None.prefix().strip_ansi(), "");
		assert_eq!(Prefix::Notice.prefix().strip_ansi(), "Notice: ");
		assert_eq!(Prefix::Success.prefix().strip_ansi(), "Success: ");
		assert_eq!(Prefix::Warning.prefix().strip_ansi(), "Warning: ");
	}

	#[test]
	fn prefix_happy() {
		assert!(Prefix::Debug.happy());
		assert!(Prefix::Info.happy());
		assert!(Prefix::None.happy());
		assert!(Prefix::Notice.happy());
		assert!(Prefix::Success.happy());

		// These two are sad.
		assert!(! Prefix::Error.happy());
		assert!(! Prefix::Warning.happy());
	}

	#[test]
	fn prefix_new() {
		assert_eq!(Prefix::new("", 199), Prefix::None);
		assert_eq!(Prefix::new("Hello", 199).prefix(), "\x1B[1;38;5;199mHello:\x1B[0m ");
		assert_eq!(Prefix::new("Hello", 2).prefix(), "\x1B[1;38;5;2mHello:\x1B[0m ");
	}

	#[test]
	fn msg_new() {
		// Just a message.
		let msg: Msg = Msg::new("Hello World");
		assert_eq!(msg.msg(), "\x1B[1mHello World\x1B[0m");
		assert_eq!(msg.prefix(), Prefix::None);

		// With prefix.
		let msg: Msg = Msg::new("Hello World")
			.with_prefix(Prefix::Success);
		assert_eq!(msg.msg(), "\x1B[92;1mSuccess:\x1B[0m \x1B[1mHello World\x1B[0m");
		assert_eq!(msg.prefix(), Prefix::Success);

		// Indentation. We've tested color enough now; let's strip ANSI
		// to make this more readable.
		let msg: Msg = Msg::new("Hello World")
			.with_indent(1);
		assert_eq!(msg.msg().strip_ansi(), "    Hello World");
		let msg: Msg = Msg::new("Hello World")
			.with_indent(2);
		assert_eq!(msg.msg().strip_ansi(), "        Hello World");
	}
}
