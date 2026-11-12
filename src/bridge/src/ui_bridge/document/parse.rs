// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
#[derive(Default,Debug)]
pub struct Element {pub tag:String,pub text:String,pub attrs:Vec<(String,String)>,pub children:Vec<Element>}
impl Element{pub fn attr(&self,key:&str)->&str{self.attrs.iter().find(|x|x.0==key).map_or("",|x|x.1.as_str())}pub fn has(&self,key:&str)->bool{self.attrs.iter().any(|x|x.0==key)}}
pub fn decode(s:&str)->String{
    let mut out=String::new();let mut rest=s;
    while let Some(i)=rest.find('&'){
        out.push_str(&rest[..i]);rest=&rest[i+1..];
        let Some(j)=rest.find(';').filter(|j|*j<32)else{out.push('&');continue};
        let name=&rest[..j];
        let numeric=if let Some(n)=name.strip_prefix("#x").or_else(||name.strip_prefix("#X")){u32::from_str_radix(n,16).ok().and_then(char::from_u32)}else if let Some(n)=name.strip_prefix('#'){n.parse().ok().and_then(char::from_u32)}else{None};
        if let Some(c)=numeric{out.push(c);rest=&rest[j+1..];continue}
        let named=match name{
        "AElig" => Some("Æ"),
        "Acirc" => Some("Â"),
        "Agrave" => Some("À"),
        "Alpha" => Some("Α"),
        "Aring" => Some("Å"),
        "Atilde" => Some("Ã"),
        "Auml" => Some("Ä"),
        "Beta" => Some("Β"),
        "Ccedil" => Some("Ç"),
        "Chi" => Some("Χ"),
        "Dagger" => Some("‡"),
        "Delta" => Some("Δ"),
        "ETH" => Some("Ð"),
        "Eacute" => Some("É"),
        "Ecirc" => Some("Ê"),
        "Egrave" => Some("È"),
        "Epsilon" => Some("Ε"),
        "Eta" => Some("Η"),
        "Euml" => Some("Ë"),
        "Gamma" => Some("Γ"),
        "Iacute" => Some("Í"),
        "Icirc" => Some("Î"),
        "Igrave" => Some("Ì"),
        "Iota" => Some("Ι"),
        "Iuml" => Some("Ï"),
        "Kappa" => Some("Κ"),
        "Lambda" => Some("Λ"),
        "Mu" => Some("Μ"),
        "Ntilde" => Some("Ñ"),
        "Nu" => Some("Ν"),
        "OElig" => Some("Œ"),
        "Oacute" => Some("Ó"),
        "Ocirc" => Some("Ô"),
        "Ograve" => Some("Ò"),
        "Omega" => Some("Ω"),
        "Omicron" => Some("Ο"),
        "Oslash" => Some("Ø"),
        "Otilde" => Some("Õ"),
        "Ouml" => Some("Ö"),
        "Phi" => Some("Φ"),
        "Pi" => Some("Π"),
        "Prime" => Some("″"),
        "Psi" => Some("Ψ"),
        "Rho" => Some("Ρ"),
        "Scaron" => Some("Š"),
        "Sigma" => Some("Σ"),
        "Tau" => Some("Τ"),
        "Theta" => Some("Θ"),
        "Uacute" => Some("Ú"),
        "Ucirc" => Some("Û"),
        "Ugrave" => Some("Ù"),
        "Upsilon" => Some("Υ"),
        "Uuml" => Some("Ü"),
        "Xi" => Some("Ξ"),
        "Yacute" => Some("Ý"),
        "Yuml" => Some("Ÿ"),
        "Zeta" => Some("Ζ"),
        "acirc" => Some("â"),
        "acute" => Some("´"),
        "aelig" => Some("æ"),
        "agrave" => Some("à"),
        "alefsym" => Some("ℵ"),
        "alpha" => Some("α"),
        "amp" => Some("&"),
        "and" => Some("∧"),
        "ang" => Some("∠"),
        "apos" => Some("'"),
        "aring" => Some("å"),
        "asymp" => Some("≈"),
        "atilde" => Some("ã"),
        "auml" => Some("ä"),
        "bdquo" => Some("„"),
        "beta" => Some("β"),
        "brvbar" => Some("¦"),
        "bull" => Some("•"),
        "cap" => Some("∩"),
        "ccedil" => Some("ç"),
        "cedil" => Some("¸"),
        "cent" => Some("¢"),
        "chi" => Some("χ"),
        "circ" => Some("ˆ"),
        "clubs" => Some("♣"),
        "cong" => Some("≅"),
        "copy" => Some("©"),
        "crarr" => Some("↵"),
        "cup" => Some("∪"),
        "curren" => Some("¤"),
        "dArr" => Some("⇓"),
        "dagger" => Some("†"),
        "darr" => Some("↓"),
        "deg" => Some("°"),
        "delta" => Some("δ"),
        "diams" => Some("♦"),
        "divide" => Some("÷"),
        "eacute" => Some("é"),
        "ecirc" => Some("ê"),
        "egrave" => Some("è"),
        "empty" => Some("∅"),
        "emsp" => Some(" "),
        "ensp" => Some(" "),
        "epsilon" => Some("ε"),
        "equiv" => Some("≡"),
        "eta" => Some("η"),
        "eth" => Some("ð"),
        "euml" => Some("ë"),
        "euro" => Some("€"),
        "exist" => Some("∃"),
        "fnof" => Some("ƒ"),
        "forall" => Some("∀"),
        "frac12" => Some("½"),
        "frac14" => Some("¼"),
        "frac34" => Some("¾"),
        "frasl" => Some("⁄"),
        "gamma" => Some("γ"),
        "ge" => Some("≥"),
        "gt" => Some(">"),
        "hArr" => Some("⇔"),
        "harr" => Some("↔"),
        "hearts" => Some("♥"),
        "hellip" => Some("…"),
        "iacute" => Some("í"),
        "icirc" => Some("î"),
        "iexcl" => Some("¡"),
        "igrave" => Some("ì"),
        "image" => Some("ℑ"),
        "infin" => Some("∞"),
        "int" => Some("∫"),
        "iota" => Some("ι"),
        "iquest" => Some("¿"),
        "isin" => Some("∈"),
        "iuml" => Some("ï"),
        "kappa" => Some("κ"),
        "lArr" => Some("⇐"),
        "lambda" => Some("λ"),
        "lang" => Some("⟨"),
        "laquo" => Some("«"),
        "larr" => Some("←"),
        "lceil" => Some("⌈"),
        "ldquo" => Some("“"),
        "le" => Some("≤"),
        "lfloor" => Some("⌊"),
        "lowast" => Some("∗"),
        "loz" => Some("◊"),
        "lrm" => Some("‎"),
        "lsaquo" => Some("‹"),
        "lsquo" => Some("‘"),
        "lt" => Some("<"),
        "macr" => Some("¯"),
        "mdash" => Some("—"),
        "micro" => Some("µ"),
        "middot" => Some("·"),
        "minus" => Some("−"),
        "mu" => Some("μ"),
        "nabla" => Some("∇"),
        "nbsp" => Some(" "),
        "ndash" => Some("–"),
        "ne" => Some("≠"),
        "ni" => Some("∋"),
        "not" => Some("¬"),
        "notin" => Some("∉"),
        "nsub" => Some("⊄"),
        "ntilde" => Some("ñ"),
        "nu" => Some("ν"),
        "oacute" => Some("ó"),
        "ocirc" => Some("ô"),
        "oelig" => Some("œ"),
        "ograve" => Some("ò"),
        "oline" => Some("‾"),
        "omega" => Some("ω"),
        "omicron" => Some("ο"),
        "oplus" => Some("⊕"),
        "or" => Some("∨"),
        "ordf" => Some("ª"),
        "ordm" => Some("º"),
        "oslash" => Some("ø"),
        "otilde" => Some("õ"),
        "otimes" => Some("⊗"),
        "ouml" => Some("ö"),
        "para" => Some("¶"),
        "part" => Some("∂"),
        "permil" => Some("‰"),
        "perp" => Some("⊥"),
        "phi" => Some("φ"),
        "pi" => Some("π"),
        "piv" => Some("ϖ"),
        "plusmn" => Some("±"),
        "pound" => Some("£"),
        "prime" => Some("′"),
        "prod" => Some("∏"),
        "prop" => Some("∝"),
        "psi" => Some("ψ"),
        "quot" => Some("\""),
        "rArr" => Some("⇒"),
        "radic" => Some("√"),
        "rang" => Some("⟩"),
        "raquo" => Some("»"),
        "rarr" => Some("→"),
        "rceil" => Some("⌉"),
        "rdquo" => Some("”"),
        "real" => Some("ℜ"),
        "reg" => Some("®"),
        "rfloor" => Some("⌋"),
        "rho" => Some("ρ"),
        "rlm" => Some("‏"),
        "rsaquo" => Some("›"),
        "rsquo" => Some("’"),
        "sbquo" => Some("‚"),
        "scaron" => Some("š"),
        "sdot" => Some("⋅"),
        "sect" => Some("§"),
        "shy" => Some("­"),
        "sigma" => Some("σ"),
        "sigmaf" => Some("ς"),
        "sim" => Some("∼"),
        "spades" => Some("♠"),
        "sub" => Some("⊂"),
        "sube" => Some("⊆"),
        "sum" => Some("∑"),
        "sup" => Some("⊃"),
        "sup1" => Some("¹"),
        "sup2" => Some("²"),
        "sup3" => Some("³"),
        "supe" => Some("⊇"),
        "szlig" => Some("ß"),
        "tau" => Some("τ"),
        "there4" => Some("∴"),
        "theta" => Some("θ"),
        "thetasym" => Some("ϑ"),
        "thinsp" => Some(" "),
        "thorn" => Some("þ"),
        "tilde" => Some("˜"),
        "times" => Some("×"),
        "trade" => Some("™"),
        "uArr" => Some("⇑"),
        "uacute" => Some("ú"),
        "uarr" => Some("↑"),
        "ucirc" => Some("û"),
        "ugrave" => Some("ù"),
        "uml" => Some("¨"),
        "upsih" => Some("ϒ"),
        "upsilon" => Some("υ"),
        "uuml" => Some("ü"),
        "weierp" => Some("℘"),
        "xi" => Some("ξ"),
        "yacute" => Some("ý"),
        "yen" => Some("¥"),
        "yuml" => Some("ÿ"),
        "zeta" => Some("ζ"),
        "zwj" => Some("‍"),
        "zwnj" => Some("‌"),
        _=>None};
        if let Some(v)=named{out.push_str(v);rest=&rest[j+1..]}else{out.push('&')}
    }
    out.push_str(rest);out
}
fn tag(s:&str)->Element{
    let mut e=Element::default();let mut p=0;let b=s.as_bytes();
    while p<b.len()&&!b[p].is_ascii_whitespace()&&b[p]!=b'/'{p+=1}
    e.tag=s[..p].to_ascii_lowercase();
    while p<b.len(){
        while p<b.len()&&(b[p].is_ascii_whitespace()||b[p]==b'/'){p+=1}let a=p;
        while p<b.len()&&!b[p].is_ascii_whitespace()&&b[p]!=b'='&&b[p]!=b'/'{p+=1}
        if a==p{break}let key=s[a..p].to_ascii_lowercase();
        while p<b.len()&&b[p].is_ascii_whitespace(){p+=1}let mut value=String::new();
        if p<b.len()&&b[p]==b'='{
            p+=1;while p<b.len()&&b[p].is_ascii_whitespace(){p+=1}
            if p<b.len(){let q=b[p];if q==b'\''||q==b'"'{p+=1;let start=p;while p<b.len()&&b[p]!=q{p+=1}value=decode(&s[start..p]);if p<b.len(){p+=1}}
            else{let start=p;while p<b.len()&&!b[p].is_ascii_whitespace(){p+=1}value=decode(&s[start..p])}}
        }
        e.attrs.push((key,value));
    }e
}
pub fn parse(s:&str)->Result<Element,String>{
    if s.len()>2*1024*1024{return Err("Document exceeds 2 MiB".into())}
    let mut stack=vec![Element{tag:"root".into(),..Default::default()}];let mut p=0;let mut count=0;
    while p<s.len(){
        count+=1;if count>30000{return Err("Document exceeds 30000 elements".into())}
        if s[p..].starts_with("<!--"){p=s[p+4..].find("-->").map_or(s.len(),|n|p+4+n+3);continue}
        if s.as_bytes()[p]!=b'<'{
            let end=s[p..].find('<').map_or(s.len(),|n|p+n);
            stack.last_mut().unwrap().children.push(Element{text:decode(&s[p..end]),..Default::default()});p=end;continue
        }
        let mut end=p+1;let mut quote=0;
        while end<s.len(){let c=s.as_bytes()[end];if quote!=0{if c==quote{quote=0}}else if c==b'\''||c==b'"'{quote=c}else if c==b'>'{break}end+=1}
        if end==s.len(){break}
        let raw=s[p+1..end].trim();p=end+1;
        if raw.starts_with('!')||raw.starts_with('?'){continue}
        if let Some(close)=raw.strip_prefix('/'){
            if let Some(at)=stack.iter().rposition(|e|e.tag==close.to_ascii_lowercase()){
                while stack.len()>at+1{let n=stack.pop().unwrap();stack.last_mut().unwrap().children.push(n)}
                if stack.len()>1{let n=stack.pop().unwrap();stack.last_mut().unwrap().children.push(n)}
            }continue
        }
        let e=tag(raw);let void=matches!(e.tag.as_str(),"br"|"hr"|"img"|"input"|"meta"|"link")||raw.ends_with('/');
        if void{stack.last_mut().unwrap().children.push(e)}else{if stack.len()>64{return Err("Document nesting exceeds 64".into())}stack.push(e)}
    }
    while stack.len()>1{let n=stack.pop().unwrap();stack.last_mut().unwrap().children.push(n)}
    Ok(stack.pop().unwrap())
}
