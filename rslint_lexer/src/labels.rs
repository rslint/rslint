use crate::{Lexer, SyntaxKind, T};

impl Lexer<'_> {
    #[inline]
    pub(crate) fn resolve_label_a(&mut self) -> Option<SyntaxKind> {
        if let Some(b"wait") = self.bytes.get(self.cur + 1..self.cur + 5) {
            self.advance(4);
            Some(T![await])
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn resolve_label_b(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur..(self.cur + 5)) {
            Some(b"break") => {
                self.advance(4);
                Some(T![break])
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_c(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'a') => match self.bytes.get(self.cur + 2) {
                Some(b't') => {
                    if let Some(b"ch") = self.bytes.get((self.cur + 3)..(self.cur + 5)) {
                        self.advance(4);
                        Some(T![catch])
                    } else {
                        None
                    }
                }
                Some(b's') => {
                    if let Some(b'e') = self.bytes.get(self.cur + 3) {
                        self.advance(3);
                        Some(T![case])
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Some(b'o') => match self.bytes.get(self.cur + 2) {
                Some(b'n') => match self.bytes.get(self.cur + 3) {
                    Some(b's') => {
                        if let Some(b't') = self.bytes.get(self.cur + 4) {
                            self.advance(4);
                            Some(T![const])
                        } else {
                            None
                        }
                    }
                    Some(b't') => {
                        if let Some(b"inue") = self.bytes.get(self.cur + 4..self.cur + 8) {
                            self.advance(7);
                            Some(T![continue])
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            },
            Some(b'l') => {
                if let Some(b"class") = self.bytes.get(self.cur..self.cur + 5) {
                    self.advance(4);
                    Some(T![class])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_d(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'e') => match self.bytes.get(self.cur + 2) {
                Some(b'b') => {
                    if let Some(b"ugger") = self.bytes.get(self.cur + 3..self.cur + 8) {
                        self.advance(7);
                        Some(T![debugger])
                    } else {
                        None
                    }
                }
                Some(b'f') => {
                    if let Some(b"ault") = self.bytes.get(self.cur + 3..self.cur + 7) {
                        self.advance(6);
                        Some(T![default])
                    } else {
                        None
                    }
                }
                Some(b'l') => {
                    if let Some(b"ete") = self.bytes.get(self.cur + 3..self.cur + 6) {
                        self.advance(5);
                        Some(T![delete])
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Some(b'o') => {
                self.advance(1);
                Some(T![do])
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_e(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'l') => {
                if let Some(b"se") = self.bytes.get(self.cur + 2..self.cur + 4) {
                    self.advance(3);
                    Some(T![else])
                } else {
                    None
                }
            }
            Some(b'n') => {
                if let Some(b"um") = self.bytes.get(self.cur + 2..self.cur + 4) {
                    self.advance(3);
                    Some(T![enum])
                } else {
                    None
                }
            }
            Some(b'x') => match self.bytes.get(self.cur + 2) {
                Some(b'p') => {
                    if let Some(b"ort") = self.bytes.get(self.cur + 3..self.cur + 6) {
                        self.advance(5);
                        Some(T![export])
                    } else {
                        None
                    }
                }
                Some(b't') => {
                    if let Some(b"ends") = self.bytes.get(self.cur + 3..self.cur + 7) {
                        self.advance(6);
                        Some(T![extends])
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_f(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'a') => {
                if let Some(b"lse") = self.bytes.get(self.cur + 2..self.cur + 5) {
                    self.advance(4);
                    Some(T![false])
                } else {
                    None
                }
            }
            Some(b'i') => {
                if let Some(b"nally") = self.bytes.get(self.cur + 2..self.cur + 7) {
                    self.advance(6);
                    Some(T![finally])
                } else {
                    None
                }
            }
            Some(b'o') => {
                if let Some(b'r') = self.bytes.get(self.cur + 2) {
                    self.advance(2);
                    Some(T![for])
                } else {
                    None
                }
            }
            Some(b'u') => {
                if let Some(b"nction") = self.bytes.get(self.cur + 2..self.cur + 8) {
                    self.advance(7);
                    Some(T![function])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_i(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'n') => {
                if let Some(b"stanceof") = self.bytes.get(self.cur + 2..self.cur + 10) {
                    self.advance(9);
                    Some(T![instanceof])
                } else {
                    self.advance(1);
                    Some(T![in])
                }
            }
            Some(b'm') => {
                if let Some(b"port") = self.bytes.get(self.cur + 2..self.cur + 6) {
                    self.advance(5);
                    Some(T![import])
                } else {
                    self.advance(1);
                    Some(T![in])
                }
            }
            Some(b'f') => {
                self.advance(1);
                Some(T![if])
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_n(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'u') => {
                if let Some(b"ll") = self.bytes.get(self.cur + 2..self.cur + 4) {
                    self.advance(3);
                    Some(T![null])
                } else {
                    None
                }
            }
            Some(b'e') => {
                if let Some(b'w') = self.bytes.get(self.cur + 2) {
                    self.advance(2);
                    Some(T![new])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_r(&mut self) -> Option<SyntaxKind> {
        if let Some(b"return") = self.bytes.get(self.cur..self.cur + 6) {
            self.advance(5);
            Some(T![return])
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn resolve_label_s(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'u') => {
                if let Some(b"per") = self.bytes.get(self.cur + 2..self.cur + 5) {
                    self.advance(4);
                    Some(T![super])
                } else {
                    None
                }
            }
            Some(b'w') => {
                // Dont mind this :)
                if let Some(b"itch") = self.bytes.get(self.cur + 2..self.cur + 6) {
                    self.advance(5);
                    Some(T![switch])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_t(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'r') => match self.bytes.get(self.cur + 2) {
                Some(b'y') => {
                    self.advance(2);
                    Some(T![try])
                }
                Some(b'u') => {
                    if let Some(b'e') = self.bytes.get(self.cur + 3) {
                        self.advance(3);
                        Some(T![true])
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Some(b'h') => match self.bytes.get(self.cur + 2) {
                Some(b'i') => {
                    if let Some(b's') = self.bytes.get(self.cur + 3) {
                        self.advance(3);
                        Some(T![this])
                    } else {
                        None
                    }
                }
                Some(b'r') => {
                    if let Some(b"ow") = self.bytes.get(self.cur + 3..self.cur + 5) {
                        self.advance(4);
                        Some(T![throw])
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Some(b'y') => {
                if let Some(b"peof") = self.bytes.get(self.cur + 2..self.cur + 6) {
                    self.advance(5);
                    Some(T![typeof])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_v(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'a') => {
                if let Some(b'r') = self.bytes.get(self.cur + 2) {
                    self.advance(2);
                    Some(T![var])
                } else {
                    None
                }
            }
            Some(b'o') => {
                if let Some(b"id") = self.bytes.get(self.cur + 2..self.cur + 4) {
                    self.advance(3);
                    Some(T![void])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn resolve_label_w(&mut self) -> Option<SyntaxKind> {
        match self.bytes.get(self.cur + 1) {
            Some(b'h') => {
                if let Some(b"ile") = self.bytes.get(self.cur + 2..self.cur + 5) {
                    self.advance(4);
                    Some(T![while])
                } else {
                    None
                }
            }
            Some(b'i') => {
                if let Some(b"th") = self.bytes.get(self.cur + 2..self.cur + 4) {
                    self.advance(3);
                    Some(T![with])
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
