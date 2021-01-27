/*!
# FYI Witcher Example: Walk and Count

This simply finds all files under /usr/share and reports the total.
*/



/// Do it.
fn main() {
	let len: usize = fyi_witcher::witch(&["/usr/share"]).len();
	println!("There are {} files in /usr/share.", len);
}
