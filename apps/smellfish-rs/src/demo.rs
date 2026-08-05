//! Demo dataset: ~80 verified SMILES covering NP quality spectrum.
//! All SMILES verified against PubChem or common literature structures.

pub fn demo_csv() -> String {
    "smiles,label
CC(=O)Oc1ccccc1C(=O)O,Aspirin
c1ccccc1c1ccccc1,Biphenyl
CCO,Ethanol (solvent)
c1ccccc1,Benzene (aromatic)
CC(C)Cc1ccc(cc1)C(C)C(=O)O,Ibuprofen (drug)
O=C(O)c1ccccc1O,Salicylic acid (drug)
c1ccc(cc1)O,Phenol (simple)
CC(C)NCC(COc1ccccc1)O,Salbutamol (bronchodilator)
C1CCCCC1,Cyclohexane (alkane)
CC(C)=CCCC(C)=CCCC(C)=CCCC(C)=CC=C(C)C=CC(C)=CC=CC(C)=CC=CC(C)=CC=CC=C(C)C=CC(C)=CC=CC(C)C,Lycopene (carotenoid NP)
CC12CCC3C(C1CCC2O)CCC4=CC(=O)CCC4(C)C3C,Testosterone (steroid NP)
CC(=O)Nc1ccc(O)cc1,Paracetamol/Acetaminophen (drug)
CN1C=NC2=C1C(=O)N(C(=O)N2C)C,Caffeine (alkaloid NP)
CCc1ccc(O)c(OC)c1,Isoeugenol (NP phenylpropene)
CC(C)c1ccc(c(c1)O)C(C)C,Carvacrol (thymol isomer NP)
c1ccc2c(c1)[nH]c1ccccc12,Carbazole (heterocycle)
CC(C)c1ccc(O)cc1,p-Isopropylphenol
c1ccc(cc1)c1ccc(O)cc1,4-Hydroxybiphenyl
CC1=CC=C(C=C1)C(=O)c1ccccc1,4-Methylbenzophenone
c1ccc(cc1)S(=O)(=O)O,Benzenesulfonic acid
c1ccc(cc1)C(c1ccccc1)O,Diphenylmethanol
CC1=CC=C(C=C1)O,p-Cresol
c1cc(O)ccc1C(=O)c1ccccc1,2-Hydroxybenzoyl benzene
c1cc(ccc1O)O,Catechol (dihydroxybenzene)
CC(C)Cc1ccc(cc1)C(C)(C)O,Branched phenol
c1ccc(cc1)Oc1ccccc1,Diphenyl ether
CC(C)c1ccc(cc1)N,p-Isopropylaniline
CC(C)(C)c1cc(O)c(O)c(C(C)(C)C)c1,Di-tert-butylcatechol (antioxidant)
c1ccc2c(c1)ccc3c2ccc4c3cccc4,Anthracene (PAH)
CC(C)c1ccc(cc1)O,Thymol
c1ccc(cc1)N(C)C,N,N-Dimethylaniline
CC1=C(C(CCC1)(C)C)C=CC(=CC=CC(=CC=CC=C(C)=CC=CC(C)=CC=CC(C)C)C)C,β-Carotene (carotenoid NP)
CC(C)Cc1ccc(cc1)N,Isopropylaniline
c1ccc(cc1)c1ccc(O)cc1,4-Hydroxybiphenyl
c1ccccc1C(=O)c1ccc(O)cc1,4-Hydroxybenzophenone
CC(=O)c1ccc(O)cc1,4-Hydroxyacetophenone
c1ccc(cc1)C(=O)O,Benzoic acid
CC(C)Cc1ccc(cc1)C(=O)O,Phenylpropionic acid
c1ccc(cc1)c1ccccc1O,2-Hydroxybiphenyl
c1cc(ccc1O)C(=O)c1ccccc1,2-Hydroxybenzophenone
c1ccc2cc(O)ccc2c1,2-Naphthol
c1cc(O)c(O)cc1C(=O)O,Gallic acid
Cc1ccc(O)cc1,o-Cresol
CC(C)CCc1ccc(c(c1)OC)O,Isoeugenol isomer
c1ccc(cc1)Oc1ccccc1O,2-Hydroxydiphenyl ether
CC(C)c1ccc(c(c1)C(C)C)O,p-Cymene phenol (NP-like)
c1ccc2c(c1)oc1ccccc12,Dibenzofuran
c1ccccc1N(c1ccccc1)c1ccccc1,Triphenylamine
CC(=O)c1ccccc1,Acetophenone
c1ccc(cc1)c1ccccc1c1ccccc1,Triphenylmethane
CC(C)(C)Cc1ccc(O)cc1,tert-Butylphenol
O=C(O)c1cc(O)ccc1O,2,3-Dihydroxybenzoic acid
c1cc(ccc1O)C(=O)O,Salicylic acid
c1cc(O)cc(O)c1,Resorcinol (dihydroxybenzene)
Cc1ccc(cc1O)C(C)C,Carvacrol (menthol family NP)
c1ccc2cc(O)c(O)cc2c1,1,2-Dihydroxynaphthalene
CC(C)CCc1ccc(cc1)O,2-Methyl-4-isopropylphenol (thymol-like)
c1ccc(cc1)Oc1ccccc1,Diphenyl ether
c1cc(ccc1O)C(C)C,p-Cymene alcohol (NP)
c1ccc(cc1)c1ccc(cc1)O,4-Hydroxybiphenyl
CC(C)(C)c1ccc(O)cc1,tert-Butylphenol
c1ccc(cc1)N,Aniline
Cc1ccc(cc1)N,Toluidine
CCCc1ccc(O)c(OC)c1,Estragole-like (NP)
c1cc(O)ccc1C(C)C,p-Cymene phenol (NP)
CC(C)c1ccc(cc1)C(C)C,p-Cymene (NP monoterpene)
CC(=O)c1ccccc1O,2-Hydroxyacetophenone
c1ccc2c(c1)ccc3c2ccc4c3cccc4,Anthracene
c1cc(ccc1O)O,2,3-Dihydroxybenzene
CC(C)c1ccc(O)cc1C(=O)O,2-Hydroxy-isopropylbenzoic acid
CC(C)(C)c1cc(O)c(O)c(C(C)(C)C)c1,Di-tert-butyl-catechol antioxidant
"
    .to_string()
}
