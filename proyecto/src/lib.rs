use anchor_lang::prelude::*;

declare_id!("HohyXMsSH2cYcKyEBFmuMoHwyMMESVBBLzNGEr2ixcSZ");

#[program]
pub mod despacho_drivers {
    use super::*;

    // ==========================
    // 1️⃣ Crear Paquetería
    // ==========================
    pub fn crear_paqueteria(
        ctx: Context<CrearPaqueteria>,
        nombre: String,
    ) -> Result<()> {
        require!(nombre.len() > 0, ErrorCode::NombreInvalido);
        require!(nombre.len() <= 64, ErrorCode::NombreMuyLargo);

        let paqueteria = &mut ctx.accounts.paqueteria;
        paqueteria.nombre = nombre;
        paqueteria.owner = ctx.accounts.owner.key();

        Ok(())
    }

    // ==========================
    // 2️⃣ Check-In Driver
    // ==========================
    pub fn check_in_driver(
        ctx: Context<CheckInDriver>,
        driver_nombre: String,
        paquetes_asignados: u32,
    ) -> Result<()> {
        require!(driver_nombre.len() > 0, ErrorCode::NombreInvalido);
        require!(driver_nombre.len() <= 64, ErrorCode::NombreMuyLargo);

        let clock = Clock::get()?;
        let sesion = &mut ctx.accounts.sesion;

        sesion.paqueteria = ctx.accounts.paqueteria.key();
        sesion.driver_nombre = driver_nombre;
        sesion.hora_entrada = clock.unix_timestamp;
        sesion.hora_salida = None;
        sesion.paquetes_asignados = paquetes_asignados;
        sesion.paquetes_finales = None;
        sesion.paquetes_retirados = None;
        sesion.tiempo_carga_segundos = None;
        sesion.activo = true;

        Ok(())
    }

    // ==========================
    // 3️⃣ Check-Out Driver
    // ==========================
    pub fn check_out_driver(
        ctx: Context<CheckOutDriver>,
        paquetes_finales: u32,
        paquetes_retirados: u32,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let sesion = &mut ctx.accounts.sesion;

        require!(sesion.activo, ErrorCode::SesionYaCerrada);

        let hora_salida = clock.unix_timestamp;
        let tiempo = hora_salida - sesion.hora_entrada;

        // Límite de 30 minutos = 1800 segundos
        require!(tiempo <= 1800, ErrorCode::TiempoCargaExcedido);

        sesion.hora_salida = Some(hora_salida);
        sesion.paquetes_finales = Some(paquetes_finales);
        sesion.paquetes_retirados = Some(paquetes_retirados);
        sesion.tiempo_carga_segundos = Some(tiempo);
        sesion.activo = false;

        Ok(())
    }
}

// ==========================
// Accounts
// ==========================

#[derive(Accounts)]
pub struct CrearPaqueteria<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Paqueteria::SPACE
    )]
    pub paqueteria: Account<'info, Paqueteria>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CheckInDriver<'info> {
    #[account(mut, has_one = owner)]
    pub paqueteria: Account<'info, Paqueteria>,

    #[account(
        init,
        payer = owner,
        space = 8 + SesionDriver::SPACE
    )]
    pub sesion: Account<'info, SesionDriver>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CheckOutDriver<'info> {
    #[account(mut)]
    pub sesion: Account<'info, SesionDriver>,
}

// ==========================
// Data Structures
// ==========================

#[account]
pub struct Paqueteria {
    pub nombre: String,
    pub owner: Pubkey,
}

impl Paqueteria {
    pub const SPACE: usize =
        4 + 64 + // nombre
        32;      // owner
}

#[account]
pub struct SesionDriver {
    pub paqueteria: Pubkey,
    pub driver_nombre: String,

    pub hora_entrada: i64,
    pub hora_salida: Option<i64>,

    pub paquetes_asignados: u32,
    pub paquetes_finales: Option<u32>,
    pub paquetes_retirados: Option<u32>,

    pub tiempo_carga_segundos: Option<i64>,

    pub activo: bool,
}

impl SesionDriver {
    pub const SPACE: usize =
        32 +        // paqueteria
        4 + 64 +    // driver_nombre
        8 +         // hora_entrada
        1 + 8 +     // hora_salida
        4 +         // paquetes_asignados
        1 + 4 +     // paquetes_finales
        1 + 4 +     // paquetes_retirados
        1 + 8 +     // tiempo_carga_segundos
        1;          // activo
}

// ==========================
// Errors
// ==========================

#[error_code]
pub enum ErrorCode {
    #[msg("Nombre inválido.")]
    NombreInvalido,

    #[msg("Nombre demasiado largo.")]
    NombreMuyLargo,

    #[msg("La sesión ya está cerrada.")]
    SesionYaCerrada,

    #[msg("Se excedieron los 30 minutos de carga.")]
    TiempoCargaExcedido,
}
