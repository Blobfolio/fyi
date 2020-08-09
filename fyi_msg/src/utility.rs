/*!
# FYI Msg: Utility Methods
*/

use std::slice;



#[allow(clippy::unreadable_literal)]
#[must_use]
/// Return Escape Sequence.
///
/// Convert a `u8` number into its equivalent byte string ANSI sequence.
///
/// It is worth noting that unfortunately BASH, et al, work on a 1-256 scale
/// instead of the more typical 0-255 that Rust's `u8` works on. Rather than
/// complicate the logic or require people to mentally subtract one from the
/// value they want, #256 is unsupported.
///
/// The codes are packed as `u128`s as that's a touch more efficient and
/// compact. Also worth noting, the packed state requires a Little Endian
/// to unpack; Big Endian is not supported.
///
/// # Examples
///
/// ```
/// assert_eq!(fyi_msg::utility::ansi_code_bold(199), b"\x1B[1;38;5;199m");
/// ```
pub fn ansi_code_bold(num: u8) -> &'static [u8] {
	static ANSI: [u128; 256] = [0, 132005402489276841845676827, 132010124855759711490890523, 132014847222242581136104219, 132019569588725450781317915, 132024291955208320426531611, 132029014321691190071745307, 132033736688174059716959003, 132038459054656929362172699, 132043181421139799007386395, 33792126998019396953189735195, 33793335923839011582364441371, 33794544849658626211539147547, 33795753775478240840713853723, 33796962701297855469888559899, 33798171627117470099063266075, 33799380552937084728237972251, 33800589478756699357412678427, 33801798404576313986587384603, 33803007330395928615762090779, 33792131720385879822834948891, 33793340646205494452009655067, 33794549572025109081184361243, 33795758497844723710359067419, 33796967423664338339533773595, 33798176349483952968708479771, 33799385275303567597883185947, 33800594201123182227057892123, 33801803126942796856232598299, 33803012052762411485407304475, 33792136442752362692480162587, 33793345368571977321654868763, 33794554294391591950829574939, 33795763220211206580004281115, 33796972146030821209178987291, 33798181071850435838353693467, 33799389997670050467528399643, 33800598923489665096703105819, 33801807849309279725877811995, 33803016775128894355052518171, 33792141165118845562125376283, 33793350090938460191300082459, 33794559016758074820474788635, 33795767942577689449649494811, 33796976868397304078824200987, 33798185794216918707998907163, 33799394720036533337173613339, 33800603645856147966348319515, 33801812571675762595523025691, 33803021497495377224697731867, 33792145887485328431770589979, 33793354813304943060945296155, 33794563739124557690120002331, 33795772664944172319294708507, 33796981590763786948469414683, 33798190516583401577644120859, 33799399442403016206818827035, 33800608368222630835993533211, 33801817294042245465168239387, 33803026219861860094342945563, 33792150609851811301415803675, 33793359535671425930590509851, 33794568461491040559765216027, 33795777387310655188939922203, 33796986313130269818114628379, 33798195238949884447289334555, 33799404164769499076464040731, 33800613090589113705638746907, 33801822016408728334813453083, 33803030942228342963988159259, 33792155332218294171061017371, 33793364258037908800235723547, 33794573183857523429410429723, 33795782109677138058585135899, 33796991035496752687759842075, 33798199961316367316934548251, 33799408887135981946109254427, 33800617812955596575283960603, 33801826738775211204458666779, 33803035664594825833633372955, 33792160054584777040706231067, 33793368980404391669880937243, 33794577906224006299055643419, 33795786832043620928230349595, 33796995757863235557405055771, 33798204683682850186579761947, 33799413609502464815754468123, 33800622535322079444929174299, 33801831461141694074103880475, 33803040386961308703278586651, 33792164776951259910351444763, 33793373702770874539526150939, 33794582628590489168700857115, 33795791554410103797875563291, 33797000480229718427050269467, 33798209406049333056224975643, 33799418331868947685399681819, 33800627257688562314574387995, 33801836183508176943749094171, 33803045109327791572923800347, 8650783255453730145457268677403, 8651092740463551490525993458459, 8651402225473372835594718239515, 8651711710483194180663443020571, 8652021195493015525732167801627, 8652330680502836870800892582683, 8652640165512658215869617363739, 8652949650522479560938342144795, 8653259135532300906007066925851, 8653568620542122251075791706907, 8650784464379549760086443383579, 8651093949389371105155168164635, 8651403434399192450223892945691, 8651712919409013795292617726747, 8652022404418835140361342507803, 8652331889428656485430067288859, 8652641374438477830498792069915, 8652950859448299175567516850971, 8653260344458120520636241632027, 8653569829467941865704966413083, 8650785673305369374715618089755, 8651095158315190719784342870811, 8651404643325012064853067651867, 8651714128334833409921792432923, 8652023613344654754990517213979, 8652333098354476100059241995035, 8652642583364297445127966776091, 8652952068374118790196691557147, 8653261553383940135265416338203, 8653571038393761480334141119259, 8650786882231188989344792795931, 8651096367241010334413517576987, 8651405852250831679482242358043, 8651715337260653024550967139099, 8652024822270474369619691920155, 8652334307280295714688416701211, 8652643792290117059757141482267, 8652953277299938404825866263323, 8653262762309759749894591044379, 8653572247319581094963315825435, 8650788091157008603973967502107, 8651097576166829949042692283163, 8651407061176651294111417064219, 8651716546186472639180141845275, 8652026031196293984248866626331, 8652335516206115329317591407387, 8652645001215936674386316188443, 8652954486225758019455040969499, 8653263971235579364523765750555, 8653573456245400709592490531611, 8650789300082828218603142208283, 8651098785092649563671866989339, 8651408270102470908740591770395, 8651717755112292253809316551451, 8652027240122113598878041332507, 8652336725131934943946766113563, 8652646210141756289015490894619, 8652955695151577634084215675675, 8653265180161398979152940456731, 8653574665171220324221665237787, 8650790509008647833232316914459, 8651099994018469178301041695515, 8651409479028290523369766476571, 8651718964038111868438491257627, 8652028449047933213507216038683, 8652337934057754558575940819739, 8652647419067575903644665600795, 8652956904077397248713390381851, 8653266389087218593782115162907, 8653575874097039938850839943963, 8650791717934467447861491620635, 8651101202944288792930216401691, 8651410687954110137998941182747, 8651720172963931483067665963803, 8652029657973752828136390744859, 8652339142983574173205115525915, 8652648627993395518273840306971, 8652958113003216863342565088027, 8653267598013038208411289869083, 8653577083022859553480014650139, 8650792926860287062490666326811, 8651102411870108407559391107867, 8651411896879929752628115888923, 8651721381889751097696840669979, 8652030866899572442765565451035, 8652340351909393787834290232091, 8652649836919215132903015013147, 8652959321929036477971739794203, 8653268806938857823040464575259, 8653578291948679168109189356315, 8650794135786106677119841032987, 8651103620795928022188565814043, 8651413105805749367257290595099, 8651722590815570712326015376155, 8652032075825392057394740157211, 8652341560835213402463464938267, 8652651045845034747532189719323, 8652960530854856092600914500379, 8653270015864677437669639281435, 8653579500874498782738364062491, 8650783260176096628326913891099, 8651092745185917973395638672155, 8651402230195739318464363453211, 8651711715205560663533088234267, 8652021200215382008601813015323, 8652330685225203353670537796379, 8652640170235024698739262577435, 8652949655244846043807987358491, 8653259140254667388876712139547, 8653568625264488733945436920603, 8650784469101916242956088597275, 8651093954111737588024813378331, 8651403439121558933093538159387, 8651712924131380278162262940443, 8652022409141201623230987721499, 8652331894151022968299712502555, 8652641379160844313368437283611, 8652950864170665658437162064667, 8653260349180487003505886845723, 8653569834190308348574611626779, 8650785678027735857585263303451, 8651095163037557202653988084507, 8651404648047378547722712865563, 8651714133057199892791437646619, 8652023618067021237860162427675, 8652333103076842582928887208731, 8652642588086663927997611989787, 8652952073096485273066336770843, 8653261558106306618135061551899, 8653571043116127963203786332955, 8650786886953555472214438009627, 8651096371963376817283162790683, 8651405856973198162351887571739, 8651715341983019507420612352795, 8652024826992840852489337133851, 8652334312002662197558061914907, 8652643797012483542626786695963, 8652953282022304887695511477019, 8653262767032126232764236258075, 8653572252041947577832961039131, 8650788095879375086843612715803, 8651097580889196431912337496859, 8651407065899017776981062277915, 8651716550908839122049787058971, 8652026035918660467118511840027, 8652335520928481812187236621083, 8652645005938303157255961402139, 8652954490948124502324686183195, 8653263975957945847393410964251, 8653573460967767192462135745307, 8650789304805194701472787421979, 8651098789815016046541512203035, 8651408274824837391610236984091, 8651717759834658736678961765147, 8652027244844480081747686546203, 8652336729854301426816411327259];

	// The result length varies according to the number of decimal places the
	// integer has.
	let len: usize = if num < 10 { 11 }
		else if num < 100 { 12 }
		else { 13 };

	unsafe { slice::from_raw_parts(ANSI.as_ptr().add(num as usize) as *const u8, len) }
}

#[must_use]
/// Time Number to String.
///
/// This is a simple conversion table for turning `u32` representations of
/// numbers 0–59 into double-digit strings like "00" and "59". It is faster
/// having these ready than trying to `itoa::write` them on-the-fly.
///
/// Out of range always comes back as "59".
///
/// The codes are packed as `u32`s as that's a touch more efficient and
/// compact. Also worth noting, the packed state requires a Little Endian
/// to unpack; Big Endian is not supported.
///
/// # Examples
///
/// ```
/// assert_eq!(fyi_msg::utility::time_format_dd(3), b"03");
/// assert_eq!(fyi_msg::utility::time_format_dd(10), b"10");
/// ```
pub fn time_format_dd(num: u32) -> &'static [u8] {
	static TIME: [u16; 60] = [12336, 12592, 12848, 13104, 13360, 13616, 13872, 14128, 14384, 14640, 12337, 12593, 12849, 13105, 13361, 13617, 13873, 14129, 14385, 14641, 12338, 12594, 12850, 13106, 13362, 13618, 13874, 14130, 14386, 14642, 12339, 12595, 12851, 13107, 13363, 13619, 13875, 14131, 14387, 14643, 12340, 12596, 12852, 13108, 13364, 13620, 13876, 14132, 14388, 14644, 12341, 12597, 12853, 13109, 13365, 13621, 13877, 14133, 14389, 14645];
	unsafe { slice::from_raw_parts(TIME.as_ptr().add(59.min(num as usize)) as *const u8, 2) }
}

#[must_use]
/// Whitespace Reservoir.
///
/// This method borrows whitespace from a static reference, useful for
/// quickly padding strings, etc.
///
/// Let's don't get crazy, though. A maximum of 255 spaces can be returned.
pub fn whitespace(num: usize) -> &'static [u8] {
	static WHITES: [u8; 255] = [32; 255];
	&WHITES[0..255.min(num)]
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_ansi_code_bold() {
		for i in 1..=255 {
			assert_eq!(
				ansi_code_bold(i),
				format!("\x1B[1;38;5;{}m", i).as_bytes(),
				"Ansi for {} is incorrect: {:?}",
				i,
				ansi_code_bold(i),
			);
		}
	}

	#[test]
	fn t_time_format_dd() {
		// Test the supported values.
		for i in 0..=59 {
			assert_eq!(
				time_format_dd(i),
				format!("{:02}", i).as_bytes(),
				"DD for {} is incorrect: {:?}",
				i,
				time_format_dd(i)
			);
		}

		// And make sure overflow works.
		assert_eq!(
			time_format_dd(60),
			time_format_dd(59),
			"DD for 60 is incorrect: {:?}",
			time_format_dd(60)
		);
	}

	#[test]
	fn t_whitespace() {
		assert_eq!(whitespace(0), b"");
		assert_eq!(whitespace(5), b"     ");
		assert_eq!(whitespace(10), b"          ");
	}
}
