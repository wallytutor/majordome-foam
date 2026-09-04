# -*- coding: utf-8 -*-

from pathlib import Path
from typing import Any, Self

from . import foam as _ext


class FoamDictFile:
    """ Base container class for manipulating OpenFOAM dictionary files.

    Parameters
    ----------
    inner : _ext.FoamDict | None = None
        Underlying Rust PyO3 FoamDict instance.
    """

    __slots__ = ("_inner",)

    def __init__(self, inner: Any | None = None) -> None:
        self._inner = inner if inner is not None else _ext.FoamDict()

    @classmethod
    def from_file(cls, path: str | Path) -> Self:
        """ Load and parse OpenFOAM dictionary from file.

        Parameters
        ----------
        path : str | Path
            Target dictionary file path.

        Returns
        -------
        Self
            Parsed dictionary instance.
        """
        path_str = str(path) if isinstance(path, Path) else path
        inner = _ext.FoamDict.from_file(path_str)
        return cls(inner)

    @classmethod
    def from_string(cls, content: str) -> Self:
        """ Parse OpenFOAM dictionary from string content.

        Parameters
        ----------
        content : str
            Raw OpenFOAM text contents.

        Returns
        -------
        Self
            Parsed dictionary instance.
        """
        inner = _ext.FoamDict.parse(content)
        return cls(inner)

    parse = from_string

    def to_foam(self) -> str:
        """ Serialize dictionary to canonical OpenFOAM string format.

        Returns
        -------
        str
            OpenFOAM formatted string representation.
        """
        return self._inner.to_foam()

    def save(self, path: str | Path | None = None) -> None:
        """ Save OpenFOAM dictionary to disk.

        Parameters
        ----------
        path : str | Path | None = None
            Output path location.

        Returns
        -------
        None
            File is written to disk.
        """
        if path is not None:
            path_str = str(path) if isinstance(path, Path) else path
            self._inner.save(path_str)

    def get(self, key_path: str, default: Any = None) -> Any:
        """ Get entry value by slash-separated key path.

        Parameters
        ----------
        key_path : str
            Slash-separated key location (e.g. "solvers/p/tolerance").
        default : Any = None
            Fallback value if key is not found.

        Returns
        -------
        Any
            Entry value or default.
        """
        val = self._inner.get(key_path)
        return val if val is not None else default

    def set(self, key_path: str, value: Any) -> None:
        """ Set entry value by slash-separated key path.

        Parameters
        ----------
        key_path : str
            Slash-separated key location.
        value : Any
            Value to assign.

        Returns
        -------
        None
            Updates internal AST structure.
        """
        self._inner.set(key_path, value)

    def delete(self, key_path: str) -> bool:
        """ Remove entry or block by slash-separated key path.

        Parameters
        ----------
        key_path : str
            Slash-separated key location to remove.

        Returns
        -------
        bool
            True if key was found and removed.
        """
        return self._inner.delete(key_path)

    def contains(self, key_path: str) -> bool:
        """ Check if key path exists in dictionary.

        Parameters
        ----------
        key_path : str
            Slash-separated key location.

        Returns
        -------
        bool
            True if path exists.
        """
        return self._inner.contains(key_path)

    def add_include(self, filename: str) -> None:
        """ Add #include directive to top of dictionary.

        Parameters
        ----------
        filename : str
            Include filename or path string.

        Returns
        -------
        None
            Inserts #include directive.
        """
        self._inner.add_include(filename)

    def add_include_etc(self, filename: str) -> None:
        """ Add #includeEtc directive to top of dictionary.

        Parameters
        ----------
        filename : str
            Relative OpenFOAM installation etc path string.

        Returns
        -------
        None
            Inserts #includeEtc directive.
        """
        self._inner.add_include_etc(filename)

    def keys(self) -> list[str]:
        """ Return list of top-level keys in dictionary.

        Returns
        -------
        list[str]
            List of key identifiers.
        """
        return self._inner.keys()

    def __getitem__(self, key: str) -> Any:
        return self._inner[key]

    def __setitem__(self, key: str, value: Any) -> None:
        self._inner[key] = value

    def __contains__(self, key: str) -> bool:
        return key in self._inner


class ControlDict(FoamDictFile):
    """ Strongly-typed interface for OpenFOAM controlDict files. """

    __slots__ = ()

    @property
    def solver(self) -> str | None:
        """ Get simulation solver or application name. """
        return self.get("application") or self.get("solver")

    @solver.setter
    def solver(self, name: str) -> None:
        """ Set simulation solver or application name. """
        self.set("application", name)

    @property
    def start_from(self) -> str | None:
        """ Get startFrom mode setting. """
        return self.get("startFrom")

    @start_from.setter
    def start_from(self, value: str) -> None:
        """ Set startFrom mode setting. """
        self.set("startFrom", value)

    @property
    def start_time(self) -> float | int | None:
        """ Get simulation start time. """
        return self.get("startTime")

    @start_time.setter
    def start_time(self, value: float | int) -> None:
        """ Set simulation start time. """
        self.set("startTime", value)

    @property
    def stop_at(self) -> str | None:
        """ Get stopAt mode setting. """
        return self.get("stopAt")

    @stop_at.setter
    def stop_at(self, value: str) -> None:
        """ Set stopAt mode setting. """
        self.set("stopAt", value)

    @property
    def end_time(self) -> float | int | None:
        """ Get simulation end time. """
        return self.get("endTime")

    @end_time.setter
    def end_time(self, value: float | int) -> None:
        """ Set simulation end time. """
        self.set("endTime", value)

    @property
    def delta_t(self) -> float | None:
        """ Get simulation time step deltaT. """
        return self.get("deltaT")

    @delta_t.setter
    def delta_t(self, value: float) -> None:
        """ Set simulation time step deltaT. """
        self.set("deltaT", value)

    @property
    def write_control(self) -> str | None:
        """ Get writeControl mode setting. """
        return self.get("writeControl")

    @write_control.setter
    def write_control(self, value: str) -> None:
        """ Set writeControl mode setting. """
        self.set("writeControl", value)

    @property
    def write_interval(self) -> float | int | None:
        """ Get output writeInterval setting. """
        return self.get("writeInterval")

    @write_interval.setter
    def write_interval(self, value: float | int) -> None:
        """ Set output writeInterval setting. """
        self.set("writeInterval", value)

    @property
    def purge_write(self) -> int | None:
        """ Get purgeWrite count setting. """
        return self.get("purgeWrite")

    @purge_write.setter
    def purge_write(self, value: int) -> None:
        """ Set purgeWrite count setting. """
        self.set("purgeWrite", value)

    @property
    def write_format(self) -> str | None:
        """ Get writeFormat setting (ascii/binary). """
        return self.get("writeFormat")

    @write_format.setter
    def write_format(self, value: str) -> None:
        """ Set writeFormat setting (ascii/binary). """
        self.set("writeFormat", value)

    @property
    def write_precision(self) -> int | None:
        """ Get numerical writePrecision setting. """
        return self.get("writePrecision")

    @write_precision.setter
    def write_precision(self, value: int) -> None:
        """ Set numerical writePrecision setting. """
        self.set("writePrecision", value)

    @property
    def run_time_modifiable(self) -> bool | None:
        """ Get runTimeModifiable boolean flag. """
        return self.get("runTimeModifiable")

    @run_time_modifiable.setter
    def run_time_modifiable(self, value: bool) -> None:
        """ Set runTimeModifiable boolean flag. """
        self.set("runTimeModifiable", value)


class FvSchemes(FoamDictFile):
    """ Strongly-typed interface for OpenFOAM fvSchemes files. """

    __slots__ = ()

    @property
    def ddt_schemes(self) -> Any:
        """ Get time derivative discretization ddtSchemes block. """
        return self.get("ddtSchemes")

    @property
    def grad_schemes(self) -> Any:
        """ Get gradient discretization gradSchemes block. """
        return self.get("gradSchemes")

    @property
    def div_schemes(self) -> Any:
        """ Get divergence discretization divSchemes block. """
        return self.get("divSchemes")

    @property
    def laplacian_schemes(self) -> Any:
        """ Get Laplacian discretization laplacianSchemes block. """
        return self.get("laplacianSchemes")

    @property
    def interpolation_schemes(self) -> Any:
        """ Get interpolationSchemes block. """
        return self.get("interpolationSchemes")

    @property
    def sn_grad_schemes(self) -> Any:
        """ Get surface-normal gradient snGradSchemes block. """
        return self.get("snGradSchemes")

    def set_div_scheme(self, field_name: str, scheme: str) -> None:
        """ Set divergence scheme for a specific field expression.

        Parameters
        ----------
        field_name : str
            Divergence field expression (e.g. "div(phi,U)").
        scheme : str
            Discretization scheme string (e.g. "Gauss limitedLinearV 1").

        Returns
        -------
        None
            Updates divSchemes block entry.
        """
        self.set(f"divSchemes/{field_name}", scheme)


class FvSolution(FoamDictFile):
    """ Strongly-typed interface for OpenFOAM fvSolution files. """

    __slots__ = ()

    @property
    def solvers(self) -> Any:
        """ Get linear equation solvers block. """
        return self.get("solvers")

    @property
    def pimple(self) -> Any:
        """ Get PIMPLE algorithm controls block. """
        return self.get("PIMPLE")

    @property
    def simple(self) -> Any:
        """ Get SIMPLE algorithm controls block. """
        return self.get("SIMPLE")

    @property
    def piso(self) -> Any:
        """ Get PISO algorithm controls block. """
        return self.get("PISO")

    @property
    def relaxation_factors(self) -> Any:
        """ Get equation and field relaxationFactors block. """
        return self.get("relaxationFactors")

    def set_solver(self, var_name: str, config: dict[str, Any]) -> None:
        """ Configure linear solver settings for a variable pattern.

        Parameters
        ----------
        var_name : str
            Variable pattern identifier (e.g. "p" or "(U|k|omega).*").
        config : dict[str, Any]
            Dictionary of linear solver properties (solver, tolerance, etc).

        Returns
        -------
        None
            Updates solvers block.
        """
        for key, val in config.items():
            self.set(f"solvers/{var_name}/{key}", val)

    def set_solver_option(self, var_name: str, option: str, value: Any) -> None:
        """ Set a single solver option for a field variable pattern.

        Parameters
        ----------
        var_name : str
            Variable pattern identifier (e.g. "p").
        option : str
            Option key name (e.g. "tolerance").
        value : Any
            Option value to assign.

        Returns
        -------
        None
            Updates solvers block.
        """
        self.set(f"solvers/{var_name}/{option}", value)


class SnappyHexMeshDict(FoamDictFile):
    """ Strongly-typed interface for OpenFOAM snappyHexMeshDict files. """

    __slots__ = ()

    @property
    def castellated_mesh(self) -> bool | None:
        """ Get castellatedMesh execution flag. """
        return self.get("castellatedMesh")

    @castellated_mesh.setter
    def castellated_mesh(self, value: bool) -> None:
        """ Set castellatedMesh execution flag. """
        self.set("castellatedMesh", value)

    @property
    def snap(self) -> bool | None:
        """ Get snap execution flag. """
        return self.get("snap")

    @snap.setter
    def snap(self, value: bool) -> None:
        """ Set snap execution flag. """
        self.set("snap", value)

    @property
    def add_layers(self) -> bool | None:
        """ Get addLayers execution flag. """
        return self.get("addLayers")

    @add_layers.setter
    def add_layers(self, value: bool) -> None:
        """ Set addLayers execution flag. """
        self.set("addLayers", value)

    @property
    def geometry(self) -> Any:
        """ Get geometry surfaces and regions block. """
        return self.get("geometry")

    @property
    def castellated_mesh_controls(self) -> Any:
        """ Get castellatedMeshControls block. """
        return self.get("castellatedMeshControls")

    @property
    def snap_controls(self) -> Any:
        """ Get snapControls block. """
        return self.get("snapControls")

    @property
    def add_layers_controls(self) -> Any:
        """ Get addLayersControls block. """
        return self.get("addLayersControls")

    @property
    def mesh_quality_controls(self) -> Any:
        """ Get meshQualityControls block. """
        return self.get("meshQualityControls")


class BlockMeshDict(FoamDictFile):
    """ Strongly-typed interface for OpenFOAM blockMeshDict files. """

    __slots__ = ()

    @property
    def scale(self) -> float | int | None:
        """ Get blockMesh length scale factor (convertToMeters). """
        return self.get("convertToMeters") or self.get("scale")

    @scale.setter
    def scale(self, value: float | int) -> None:
        """ Set blockMesh length scale factor (convertToMeters). """
        self.set("convertToMeters", value)

    @property
    def convert_to_meters(self) -> float | int | None:
        """ Get blockMesh length scale factor (convertToMeters). """
        return self.scale

    @convert_to_meters.setter
    def convert_to_meters(self, value: float | int) -> None:
        """ Set blockMesh length scale factor (convertToMeters). """
        self.scale = value

    @property
    def vertices(self) -> Any:
        """ Get blockMesh vertices list. """
        return self.get("vertices")

    @property
    def blocks(self) -> Any:
        """ Get hex blocks topology list. """
        return self.get("blocks")

    @property
    def edges(self) -> Any:
        """ Get curved block edges list. """
        return self.get("edges")

    @property
    def boundary(self) -> Any:
        """ Get blockMesh boundary patches list. """
        return self.get("boundary")


class DecomposeParDict(FoamDictFile):
    """ Strongly-typed interface for OpenFOAM decomposeParDict files. """

    __slots__ = ()

    @property
    def number_of_subdomains(self) -> int | None:
        """ Get numberOfSubdomains core count. """
        return self.get("numberOfSubdomains")

    @number_of_subdomains.setter
    def number_of_subdomains(self, value: int) -> None:
        """ Set numberOfSubdomains core count. """
        self.set("numberOfSubdomains", value)

    @property
    def method(self) -> str | None:
        """ Get domain decomposition method (simple, hierarchical, etc). """
        return self.get("method")

    @method.setter
    def method(self, value: str) -> None:
        """ Set domain decomposition method (simple, hierarchical, etc). """
        self.set("method", value)

    def set_simple_coeffs(self, n_vector: list[int]) -> None:
        """ Set simpleCoeffs sub-domain division vector.

        Parameters
        ----------
        n_vector : list[int]
            Division count per coordinate axis (e.g. [1, 2, 2]).

        Returns
        -------
        None
            Updates simpleCoeffs/n entry.
        """
        self.set("simpleCoeffs/n", n_vector)


class FieldFile(FoamDictFile):
    """ Strongly-typed interface for OpenFOAM initial field files (0/*). """

    __slots__ = ()

    @property
    def dimensions(self) -> list[int] | None:
        """ Get field physical dimensions array [m kg s K mol A cd]. """
        return self.get("dimensions")

    @dimensions.setter
    def dimensions(self, value: list[int]) -> None:
        """ Set field physical dimensions array. """
        self.set("dimensions", value)

    @property
    def internal_field(self) -> Any:
        """ Get internalField values block or uniform value. """
        return self.get("internalField")

    @internal_field.setter
    def internal_field(self, value: Any) -> None:
        """ Set internalField values block or uniform value. """
        self.set("internalField", value)

    @property
    def boundary_field(self) -> Any:
        """ Get boundaryField patches dictionary. """
        return self.get("boundaryField")


class VolScalarField(FieldFile):
    """ Volumetric scalar field initial conditions file. """

    __slots__ = ()


class VolVectorField(FieldFile):
    """ Volumetric vector field initial conditions file. """

    __slots__ = ()
