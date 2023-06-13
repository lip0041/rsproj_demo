#![allow(unused)]

trait UF {
    fn is_connected(&mut self, p: usize, q: usize) -> bool;
    fn union(&mut self, p: usize, q: usize);
    fn size(&self) -> usize;
}
#[derive(Debug)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>
}

impl UnionFind {
    pub fn with_capacity(size: usize) -> Self {
        let mut res = Self {
            parent: vec![0_usize; size],
            rank: vec![1_usize; size],
        };

        for i in 0..res.parent.len() {
            res.parent[i] = i;
        }
        res
    }

    fn find(&mut self, p: usize) -> Result<usize, &'static str> {
        if p >= self.parent.len() {
            return Err("param error");
        }
        let mut c = p;
        while c != self.parent[c] {
            self.parent[c] = self.parent[self.parent[c]];
            c = self.parent[c];
        }
        Ok(c)
    }
}

impl UF for UnionFind {
    fn is_connected(&mut self, p: usize, q: usize) -> bool {
        let p_root = self.find(p).unwrap();
        let q_root = self.find(q).unwrap();
        return p_root == q_root;
    }

    fn union(&mut self, p: usize, q: usize) {
        let p_root = self.find(p).unwrap();
        let q_root = self.find(q).unwrap();
        if p_root != q_root {
            if self.rank[p_root] < self.rank[q_root] {
                self.parent[p_root] = self.parent[q_root];
            } else if self.rank[p_root] > self.rank[q_root] {
                self.parent[q_root] = self.rank[p_root];
            } else {
                self.parent[q_root] = self.parent[p_root];
                self.rank[p_root] += 1;
            }
        }
    }

    fn size(&self) -> usize {
        self.parent.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uf() {
        let mut uf = UnionFind::with_capacity(10);
        uf.union(3, 5);
        uf.union(2, 1);
        uf.union(5, 1);
        uf.union(5, 4);
        println!("{:?}", uf);
        assert_eq!(uf.is_connected(4, 1), true);
    }
}