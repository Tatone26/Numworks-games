from typing import Tuple, List, Callable, Union, Optional, NoReturn

_ColorInput = Union[Tuple[int, int, int], List[int, int, int], str]

def menu(title : str,
         visible_addons : Callable[[], NoReturn],
         select_col : _ColorInput,
         bkgd_col : _ColorInput,
         list_opt : List[Tuple[str, Tuple[str, str], vars]],
         text_col : Optional[_ColorInput]=(0, 0, 0)) -> List: """Creates an entire menu. The parameters seem clear."""

def options(olist : List[Tuple[str, Tuple[str, str], vars]],
            select_col : _ColorInput,
            bkgd_col : _ColorInput,
            text_col : Optional[_ColorInput]=(0, 0, 0)) -> List: """Options menu."""

def move_select(size : int, pos : int, vis_fonc : Callable[[int, int], NoReturn]) -> int: """Select something. Is used with a function doing the visual, taking the last pos and the new one."""

def draw_centered_string(text : str,
                         posy : int,
                         color : Optional[_ColorInput]=(0, 0, 0),
                         background : Optional[_ColorInput]=(255, 255, 255)) -> NoReturn : """Like the "draw_string" function of the kandinsky module, but takes only a y coordinates and draws the text centered in the screen."""

def fill_screen(color : Tuple[int, int, int]) -> NoReturn : """Fills the screen with a unique color. """