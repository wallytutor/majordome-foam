# -*- coding: utf-8 -*-

import unittest
from pathlib import Path
from majordome_foam.foam import FoamDict
from majordome_foam import (
    BlockMeshDict,
    ControlDict,
    DecomposeParDict,
    FieldFile,
    FoamCaseHandle,
    FvSchemes,
    FvSolution,
    NotACaseError,
    SnappyHexMeshDict,
)

TUTORIALS_DIR = Path(__file__).parent.parent / "sample"


class TestMajordomeFoam(unittest.TestCase):

    def test_foam_dict_raw_rust_binding(self):
        content = """
        application     simpleFoam;
        startFrom       startTime;
        startTime       0;
        stopAt          endTime;
        endTime         1000;
        deltaT          1;
        writeControl    timeStep;
        writeInterval   50;
        purgeWrite      0;
        writeFormat     ascii;
        writePrecision  6;
        writeCompression off;
        timeFormat      general;
        timePrecision   6;
        runTimeModifiable true;
        functions
        {
            #include "foamFunction"
        }
        """
        dict_obj = FoamDict.parse(content)
        self.assertEqual(dict_obj.get("application"), "simpleFoam")
        self.assertEqual(dict_obj.get("endTime"), 1000)
        self.assertTrue(dict_obj.get("runTimeModifiable"))

        dict_obj.set("endTime", 2000)
        self.assertEqual(dict_obj.get("endTime"), 2000)

        dict_obj.set("solvers/p/tolerance", 1e-6)
        self.assertEqual(dict_obj.get("solvers/p/tolerance"), 1e-6)

        self.assertIn("application", dict_obj)
        self.assertTrue(dict_obj.delete("purgeWrite"))
        self.assertNotIn("purgeWrite", dict_obj)

    def test_to_foam_alignment_and_spacing(self):
        content = """
        application simpleFoam;
        deltaT 1;
        runTimeModifiable true;
        """
        dict_obj = FoamDict.parse(content)
        foam_str = dict_obj.to_foam()

        lines = [line for line in foam_str.splitlines() if line.strip()]
        self.assertEqual(len(lines), 3)

        self.assertTrue(lines[0].startswith("application        simpleFoam;"))
        self.assertTrue(lines[1].startswith("deltaT             1;"))
        self.assertTrue(lines[2].startswith("runTimeModifiable  true;"))

        raw_lines = foam_str.splitlines()
        self.assertEqual(raw_lines[1], "")
        self.assertEqual(raw_lines[3], "")

    def test_control_dict_wrapper(self):
        pitz_control = TUTORIALS_DIR / "01-pitzDaily/system/controlDict"
        if pitz_control.exists():
            cd = ControlDict.from_file(pitz_control)
            self.assertEqual(cd.application, "simpleFoam")
            self.assertEqual(cd.start_from, "startTime")
            self.assertEqual(cd.stop_at, "endTime")

            cd.end_time = 500.0
            self.assertEqual(cd.end_time, 500.0)

    def test_fv_schemes_wrapper(self):
        schemes_content = """
        ddtSchemes
        {
            default         steadyState;
        }
        gradSchemes
        {
            default         Gauss linear;
        }
        divSchemes
        {
            default         none;
            div(phi,U)      bounded Gauss linearUpwind grad(U);
        }
        """
        schemes = FvSchemes.parse(schemes_content)
        self.assertEqual(schemes.ddt_schemes.get("default"), "steadyState")
        self.assertEqual(schemes.grad_schemes.get("default"), "Gauss linear")

    def test_fv_solution_wrapper(self):
        sol_content = """
        solvers
        {
            p
            {
                solver          GAMG;
                tolerance       1e-06;
                relTol          0.01;
            }
        }
        SIMPLE
        {
            nCorrectors     2;
            consistent      yes;
        }
        """
        sol = FvSolution.parse(sol_content)
        self.assertEqual(sol.get("solvers/p/solver"), "GAMG")
        sol.set_solver_option("p", "tolerance", 1e-7)
        self.assertEqual(sol.get("solvers/p/tolerance"), 1e-7)

    def test_snappy_hex_mesh_dict(self):
        snappy_content = """
        castellatedMesh true;
        snap            true;
        addLayers       false;
        geometry
        {
            box.obj
            {
                type triSurfaceMesh;
                name box;
            }
        }
        """
        snappy = SnappyHexMeshDict.parse(snappy_content)
        self.assertTrue(snappy.castellated_mesh)
        self.assertTrue(snappy.snap)
        self.assertFalse(snappy.add_layers)

        snappy.add_layers = True
        self.assertTrue(snappy.add_layers)

    def test_block_mesh_dict(self):
        bm_content = """
        convertToMeters 0.001;
        """
        bm = BlockMeshDict.parse(bm_content)
        self.assertEqual(bm.convert_to_meters, 0.001)

        bm.convert_to_meters = 1.0
        self.assertEqual(bm.convert_to_meters, 1.0)

    def test_decompose_par_dict(self):
        dec_content = """
        numberOfSubdomains 4;
        method          scotch;
        """
        dec = DecomposeParDict.parse(dec_content)
        self.assertEqual(dec.number_of_subdomains, 4)
        self.assertEqual(dec.method, "scotch")

        dec.number_of_subdomains = 8
        self.assertEqual(dec.number_of_subdomains, 8)

    def test_field_file(self):
        p_content = """
        dimensions      [0 2 -2 0 0 0 0];
        internalField   uniform 0;
        boundaryField
        {
            inlet
            {
                type            zeroGradient;
            }
        }
        """
        field = FieldFile.parse(p_content)
        self.assertEqual(field.dimensions, [0, 2, -2, 0, 0, 0, 0])
        self.assertEqual(field.internal_field, "uniform 0")

        field.dimensions = [1, -1, -2, 0, 0, 0, 0]
        self.assertEqual(field.dimensions, [1, -1, -2, 0, 0, 0, 0])

    def test_foam_case_handle(self):
        pitz_dir = TUTORIALS_DIR / "01-pitzDaily"
        if pitz_dir.exists():
            case = FoamCaseHandle(root_dir=pitz_dir)
            self.assertTrue(case.is_valid)

            cd = case.controlDict
            self.assertEqual(cd.application, "simpleFoam")
            self.assertEqual(cd.start_from, "startTime")

            bm = case.blockMeshDict
            self.assertIsNotNone(bm)

            schemes = case.fvSchemes
            self.assertIsNotNone(schemes)

            sol = case.fvSolution
            self.assertIsNotNone(sol)

            p_field = case.p
            self.assertEqual(p_field.dimensions, [0, 2, -2, 0, 0, 0, 0])

        invalid_case = FoamCaseHandle(root_dir=TUTORIALS_DIR)
        self.assertFalse(invalid_case.is_valid)

        with self.assertRaises(NotACaseError):
            _ = invalid_case.controlDict


if __name__ == "__main__":
    unittest.main()
